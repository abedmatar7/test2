package dev.medpulse.app;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.Service;
import android.content.Context;
import android.content.Intent;
import android.media.Ringtone;
import android.media.RingtoneManager;
import android.net.Uri;
import android.os.CountDownTimer;
import android.os.IBinder;
import android.os.PowerManager;
import android.os.VibrationEffect;
import android.os.Vibrator;

/**
 * Screen-off-safe fallback ForegroundService.
 * Android restricts third-party apps from taking over the hardware system Always-On Display (AOD).
 * While MedPulse is in the foreground, its Pure-Black (#000000) OLED Ambient Panel keeps the screen
 * awake at 5% brightness with 60-second pixel-shift burn-in protection.
 * If the user presses the physical power button to turn off the screen, this ForegroundService
 * maintains a PARTIAL_WAKE_LOCK, updates a silent lock-screen progress notification, and fires
 * an alarm + haptic vibration pattern when the focus block finishes.
 */
public class MedPulseTimerService extends Service {
    private static final String CHANNEL_ACTIVE = "medpulse_timer_active";
    private static final String CHANNEL_ALARM = "medpulse_timer_alarm";
    private PowerManager.WakeLock wakeLock;
    private CountDownTimer countDownTimer;

    @Override
    public void onCreate() {
        super.onCreate();
        NotificationManager nm = (NotificationManager) getSystemService(Context.NOTIFICATION_SERVICE);
        NotificationChannel activeChannel = new NotificationChannel(
                CHANNEL_ACTIVE,
                "Active Study Block Countdown",
                NotificationManager.IMPORTANCE_LOW
        );
        NotificationChannel alarmChannel = new NotificationChannel(
                CHANNEL_ALARM,
                "Study Block Completion Alarm",
                NotificationManager.IMPORTANCE_HIGH
        );
        nm.createNotificationChannel(activeChannel);
        nm.createNotificationChannel(alarmChannel);
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        long durationSeconds = intent != null ? intent.getLongExtra("duration_seconds", 1500L) : 1500L;
        String topicTitle = intent != null ? intent.getStringExtra("topic_title") : "Medical Study Session";

        PowerManager pm = (PowerManager) getSystemService(Context.POWER_SERVICE);
        if (wakeLock == null) {
            wakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "MedPulse::OfflineStudyTimerWakeLock");
            wakeLock.acquire(durationSeconds * 1000L + 10000L);
        }

        Notification ongoing = new Notification.Builder(this, CHANNEL_ACTIVE)
                .setContentTitle("MedPulse Focus Active: " + topicTitle)
                .setContentText("Screen-off fallback guard active")
                .setSmallIcon(android.R.drawable.ic_lock_idle_alarm)
                .setOngoing(true)
                .build();

        startForeground(1001, ongoing);

        if (countDownTimer != null) {
            countDownTimer.cancel();
        }
        countDownTimer = new CountDownTimer(durationSeconds * 1000L, 1000L) {
            @Override
            public void onTick(long millisUntilFinished) {
                // Wall-clock reconciliation in Rust syncs exact second on screen wake
            }

            @Override
            public void onFinish() {
                triggerCompletionAlarm(topicTitle);
                stopSelf();
            }
        }.start();

        return START_STICKY;
    }

    private void triggerCompletionAlarm(String topicTitle) {
        Vibrator vib = (Vibrator) getSystemService(Context.VIBRATOR_SERVICE);
        if (vib != null && vib.hasVibrator()) {
            long[] pattern = new long[]{0, 400, 200, 400, 200, 600};
            vib.vibrate(VibrationEffect.createWaveform(pattern, -1));
        }
        try {
            Uri alarmUri = RingtoneManager.getDefaultUri(RingtoneManager.TYPE_ALARM);
            if (alarmUri == null) {
                alarmUri = RingtoneManager.getDefaultUri(RingtoneManager.TYPE_NOTIFICATION);
            }
            Ringtone r = RingtoneManager.getRingtone(getApplicationContext(), alarmUri);
            if (r != null) {
                r.play();
            }
        } catch (Exception ignored) {
        }

        NotificationManager nm = (NotificationManager) getSystemService(Context.NOTIFICATION_SERVICE);
        Notification done = new Notification.Builder(this, CHANNEL_ALARM)
                .setContentTitle("Focus Block Complete — " + topicTitle)
                .setContentText("Open MedPulse to review Why This Matters & start your 20-20-20 break.")
                .setSmallIcon(android.R.drawable.ic_lock_idle_alarm)
                .setAutoCancel(true)
                .build();
        nm.notify(1002, done);
    }

    @Override
    public void onDestroy() {
        if (countDownTimer != null) {
            countDownTimer.cancel();
        }
        if (wakeLock != null && wakeLock.isHeld()) {
            wakeLock.release();
        }
        super.onDestroy();
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
}
