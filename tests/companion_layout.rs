use study_rpg::{CompanionDisplay, CompanionMode, CompanionPreferences, companion_window_bounds};

#[test]
fn expanded_companion_stays_on_the_right_and_clamps_its_vertical_position() {
    let display = CompanionDisplay {
        x: 0,
        y: 0,
        width: 1_440,
        height: 900,
        scale_factor: 1.0,
    };

    let bounds = companion_window_bounds(
        display,
        CompanionPreferences {
            mode: CompanionMode::Expanded,
            y_position: Some(800),
        },
    );

    assert_eq!(bounds.x, 1_080);
    assert_eq!(bounds.y, 264);
    assert_eq!(bounds.width, 360);
    assert_eq!(bounds.height, 620);
}
