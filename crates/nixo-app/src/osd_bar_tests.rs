#[cfg(test)]
mod tests {
    use crate::osd_bar::OsdTier;

    #[test]
    fn test_tier_from_width() {
        assert_eq!(OsdTier::from_width(1920.0), OsdTier::Full);
        assert_eq!(OsdTier::from_width(800.0), OsdTier::Full);
        assert_eq!(OsdTier::from_width(780.0), OsdTier::Full);
        assert_eq!(OsdTier::from_width(779.0), OsdTier::Medium);
        assert_eq!(OsdTier::from_width(600.0), OsdTier::Medium);
        assert_eq!(OsdTier::from_width(580.0), OsdTier::Medium);
        assert_eq!(OsdTier::from_width(579.0), OsdTier::Compact);
        assert_eq!(OsdTier::from_width(450.0), OsdTier::Compact);
        assert_eq!(OsdTier::from_width(420.0), OsdTier::Compact);
        assert_eq!(OsdTier::from_width(419.0), OsdTier::Mini);
        assert_eq!(OsdTier::from_width(320.0), OsdTier::Mini);
    }

    #[test]
    fn test_tier_right_button_counts() {
        assert_eq!(OsdTier::Full.right_button_count(), 7);
        assert_eq!(OsdTier::Medium.right_button_count(), 5);
        assert_eq!(OsdTier::Compact.right_button_count(), 3);
        assert_eq!(OsdTier::Mini.right_button_count(), 2);
    }

    #[test]
    fn test_tier_feature_flags() {
        assert!(OsdTier::Full.show_aspect_and_screenshot());
        assert!(!OsdTier::Medium.show_aspect_and_screenshot());

        assert!(OsdTier::Medium.show_repeat_and_home());
        assert!(!OsdTier::Compact.show_repeat_and_home());

        assert!(OsdTier::Compact.show_cc());
        assert!(!OsdTier::Mini.show_cc());

        assert!(OsdTier::Compact.show_track_navigation());
        assert!(!OsdTier::Mini.show_track_navigation());
    }
}
