use crate::models::{Quote, QuoteTheme};

pub struct QuoteDeck {
    pub quotes: Vec<Quote>,
}

impl QuoteDeck {
    pub fn from_json_str(raw: &str) -> Result<Self, serde_json::Error> {
        let quotes: Vec<Quote> = serde_json::from_str(raw)?;
        Ok(Self { quotes })
    }

    pub fn total_count(&self) -> usize {
        self.quotes.len()
    }

    pub fn remaining_unseen_in_cycle(&self) -> usize {
        self.quotes
            .iter()
            .filter(|q| !q.shown_in_current_cycle)
            .count()
    }

    /// Selects the next quote without ever repeating until all 150+ quotes in the deck
    /// have been shown. Optionally prefers a theme if unseen quotes exist in that theme.
    pub fn draw_next_quote(&mut self, preferred_theme: Option<QuoteTheme>) -> Option<Quote> {
        if self.quotes.is_empty() {
            return None;
        }

        // If all quotes have been shown in the current cycle, reset the cycle automatically.
        if self.quotes.iter().all(|q| q.shown_in_current_cycle) {
            for q in &mut self.quotes {
                q.shown_in_current_cycle = false;
            }
        }

        let target_idx = if let Some(theme) = preferred_theme {
            self.quotes
                .iter()
                .position(|q| !q.shown_in_current_cycle && q.theme == theme)
                .or_else(|| self.quotes.iter().position(|q| !q.shown_in_current_cycle))
        } else {
            self.quotes.iter().position(|q| !q.shown_in_current_cycle)
        };

        if let Some(idx) = target_idx {
            self.quotes[idx].shown_in_current_cycle = true;
            Some(self.quotes[idx].clone())
        } else {
            None
        }
    }

    pub fn toggle_favorite(&mut self, quote_id: &str) -> Option<bool> {
        if let Some(q) = self.quotes.iter_mut().find(|q| q.id == quote_id) {
            q.is_favorite = !q.is_favorite;
            Some(q.is_favorite)
        } else {
            None
        }
    }

    pub fn favorites(&self) -> Vec<&Quote> {
        self.quotes.iter().filter(|q| q.is_favorite).collect()
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceBasedBooster {
    pub id: &'static str,
    pub category: &'static str, // "Study Science" or "Micro-Break"
    pub title: &'static str,
    pub actionable_tip: &'static str,
}

pub const ROTATING_STUDY_TIPS: &[EvidenceBasedBooster] = &[
    EvidenceBasedBooster {
        id: "retrieval_practice",
        category: "Study Science",
        title: "Retrieval Practice Over Re-Reading",
        actionable_tip: "Close the Robbins chapter or notes and reconstruct the diagnostic cascade from memory before checking.",
    },
    EvidenceBasedBooster {
        id: "spaced_repetition",
        category: "Study Science",
        title: "Synaptic Spacing Effect",
        actionable_tip: "Reviewing right before the forgetting curve dips (around 90% retrievability) maximizes long-term stability per minute spent.",
    },
    EvidenceBasedBooster {
        id: "interleaving",
        category: "Study Science",
        title: "Interleaved Differential Discrimination",
        actionable_tip: "Mix adjacent presentations (e.g., nephritic vs. nephrotic syndromes, or restrictive vs. obstructive lung disease) rather_than blocking one topic for hours.",
    },
    EvidenceBasedBooster {
        id: "feynman_elaboration",
        category: "Study Science",
        title: "Feynman Bedside Translation",
        actionable_tip: "If you can explain the pathophysiology in 2 plain sentences a frightened patient understands, you own the mechanism.",
    },
    EvidenceBasedBooster {
        id: "case_based_transfer",
        category: "Study Science",
        title: "Case-Based Illness Scripts",
        actionable_tip: "Anchor every disease around 3 nodes: Epidemiology/Risk Factors, Pathophysiological Mechanism, and Defining Clinical/Histologic Sign.",
    },
    EvidenceBasedBooster {
        id: "practice_questions",
        category: "Study Science",
        title: "Diagnostic Error Analysis",
        actionable_tip: "When reviewing a vignette, write down what single finding in the stem would make each wrong option become the right answer.",
    },
];

pub const BREAK_SUGGESTIONS: &[EvidenceBasedBooster] = &[
    EvidenceBasedBooster {
        id: "rule_20_20_20",
        category: "Micro-Break",
        title: "20-20-20 Ciliary Muscle Reset",
        actionable_tip: "Look at an object at least 20 feet away for 20 seconds and blink slowly to relax accommodation and restore tear film.",
    },
    EvidenceBasedBooster {
        id: "hydration_electrolytes",
        category: "Micro-Break",
        title: "Hydration & Preload Check",
        actionable_tip: "Drink 200 mL of water now—even 1–2% dehydration measurably impairs working memory and executive attention.",
    },
    EvidenceBasedBooster {
        id: "thoracic_extension",
        category: "Micro-Break",
        title: "Suboccipital & Thoracic Unclench",
        actionable_tip: "Stand up, roll scapulae back and down, and perform two physiological sighs (double nasal inhale, long slow mouth exhale).",
    },
];
