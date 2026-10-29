use super::*;
use crate::match_arg::MatchArg;

#[derive(Copy, Clone, Debug, PartialEq)]
enum TestColor {
    Red,
    Green,
    Blue,
}

impl MatchArg for TestColor {
    const CHOICES: &'static [&'static str] = &["Red", "Green", "Blue"];

    fn from_choice(choice: &str) -> Option<Self> {
        match choice {
            "Red" => Some(TestColor::Red),
            "Green" => Some(TestColor::Green),
            "Blue" => Some(TestColor::Blue),
            _ => None,
        }
    }

    fn to_choice(self) -> &'static str {
        match self {
            TestColor::Red => "Red",
            TestColor::Green => "Green",
            TestColor::Blue => "Blue",
        }
    }
}

impl RFactor for TestColor {
    fn to_level_index(self) -> i32 {
        match self {
            TestColor::Red => 1,
            TestColor::Green => 2,
            TestColor::Blue => 3,
        }
    }

    fn from_level_index(idx: i32) -> Option<Self> {
        match idx {
            1 => Some(TestColor::Red),
            2 => Some(TestColor::Green),
            3 => Some(TestColor::Blue),
            _ => None,
        }
    }
}

#[test]
fn test_level_index_roundtrip() {
    assert_eq!(
        TestColor::from_level_index(TestColor::Red.to_level_index()),
        Some(TestColor::Red)
    );
    assert_eq!(
        TestColor::from_level_index(TestColor::Green.to_level_index()),
        Some(TestColor::Green)
    );
    assert_eq!(
        TestColor::from_level_index(TestColor::Blue.to_level_index()),
        Some(TestColor::Blue)
    );
}

#[test]
fn test_invalid_index() {
    assert_eq!(TestColor::from_level_index(0), None);
    assert_eq!(TestColor::from_level_index(4), None);
    assert_eq!(TestColor::from_level_index(-1), None);
}

#[test]
fn test_levels_array() {
    assert_eq!(TestColor::CHOICES, &["Red", "Green", "Blue"]);
}

// Test interaction factor (manual impl to verify logic)
#[derive(Copy, Clone, Debug, PartialEq)]
enum Size {
    Small,
    Large,
}

impl MatchArg for Size {
    const CHOICES: &'static [&'static str] = &["Small", "Large"];

    fn from_choice(choice: &str) -> Option<Self> {
        match choice {
            "Small" => Some(Size::Small),
            "Large" => Some(Size::Large),
            _ => None,
        }
    }

    fn to_choice(self) -> &'static str {
        match self {
            Size::Small => "Small",
            Size::Large => "Large",
        }
    }
}

impl RFactor for Size {
    fn to_level_index(self) -> i32 {
        match self {
            Size::Small => 1,
            Size::Large => 2,
        }
    }

    fn from_level_index(idx: i32) -> Option<Self> {
        match idx {
            1 => Some(Size::Small),
            2 => Some(Size::Large),
            _ => None,
        }
    }
}

// Manual interaction factor impl (what derive should generate)
#[derive(Copy, Clone, Debug, PartialEq)]
enum ColorSize {
    Red(Size),
    Green(Size),
    Blue(Size),
}

impl MatchArg for ColorSize {
    const CHOICES: &'static [&'static str] = &[
        "Red.Small",
        "Red.Large",
        "Green.Small",
        "Green.Large",
        "Blue.Small",
        "Blue.Large",
    ];

    fn from_choice(choice: &str) -> Option<Self> {
        let idx_1 = Self::CHOICES
            .iter()
            .position(|&l| l == choice)
            .map(|i| i as i32 + 1)?;
        Self::from_level_index(idx_1)
    }

    fn to_choice(self) -> &'static str {
        Self::CHOICES[(self.to_level_index() - 1) as usize]
    }
}

impl RFactor for ColorSize {
    fn to_level_index(self) -> i32 {
        match self {
            Self::Red(inner) => {
                let inner_idx_0 = inner.to_level_index() - 1;
                inner_idx_0 + 1
            }
            Self::Green(inner) => {
                let inner_idx_0 = inner.to_level_index() - 1;
                2 + inner_idx_0 + 1
            }
            Self::Blue(inner) => {
                let inner_idx_0 = inner.to_level_index() - 1;
                2 * 2 + inner_idx_0 + 1
            }
        }
    }

    fn from_level_index(idx: i32) -> Option<Self> {
        match idx {
            1..=2 => {
                let inner_idx_1 = (idx - 1) % 2 + 1;
                Size::from_level_index(inner_idx_1).map(Self::Red)
            }
            3..=4 => {
                let inner_idx_1 = (idx - 1) % 2 + 1;
                Size::from_level_index(inner_idx_1).map(Self::Green)
            }
            5..=6 => {
                let inner_idx_1 = (idx - 1) % 2 + 1;
                Size::from_level_index(inner_idx_1).map(Self::Blue)
            }
            _ => None,
        }
    }
}

#[test]
fn test_interaction_levels() {
    assert_eq!(
        ColorSize::CHOICES,
        &[
            "Red.Small",
            "Red.Large",
            "Green.Small",
            "Green.Large",
            "Blue.Small",
            "Blue.Large"
        ]
    );
}

#[test]
fn test_interaction_to_index() {
    assert_eq!(ColorSize::Red(Size::Small).to_level_index(), 1);
    assert_eq!(ColorSize::Red(Size::Large).to_level_index(), 2);
    assert_eq!(ColorSize::Green(Size::Small).to_level_index(), 3);
    assert_eq!(ColorSize::Green(Size::Large).to_level_index(), 4);
    assert_eq!(ColorSize::Blue(Size::Small).to_level_index(), 5);
    assert_eq!(ColorSize::Blue(Size::Large).to_level_index(), 6);
}

#[test]
fn test_interaction_from_index() {
    assert_eq!(
        ColorSize::from_level_index(1),
        Some(ColorSize::Red(Size::Small))
    );
    assert_eq!(
        ColorSize::from_level_index(2),
        Some(ColorSize::Red(Size::Large))
    );
    assert_eq!(
        ColorSize::from_level_index(3),
        Some(ColorSize::Green(Size::Small))
    );
    assert_eq!(
        ColorSize::from_level_index(4),
        Some(ColorSize::Green(Size::Large))
    );
    assert_eq!(
        ColorSize::from_level_index(5),
        Some(ColorSize::Blue(Size::Small))
    );
    assert_eq!(
        ColorSize::from_level_index(6),
        Some(ColorSize::Blue(Size::Large))
    );
    assert_eq!(ColorSize::from_level_index(0), None);
    assert_eq!(ColorSize::from_level_index(7), None);
}

#[test]
fn test_interaction_roundtrip() {
    for i in 1..=6 {
        let color_size = ColorSize::from_level_index(i).unwrap();
        assert_eq!(color_size.to_level_index(), i);
    }
}
