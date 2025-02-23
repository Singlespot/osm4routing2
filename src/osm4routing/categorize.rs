use serde::Serialize;

pub trait Accessibility<T> {
    fn allow(&mut self, value: T);
    fn force_allow(&mut self, value: T);
    fn forbid(&mut self);
    fn soft_forbid(&mut self);
}

macro_rules! add_allow_forbid {
    ($($ty:ty),*) => {
        $(impl Accessibility<$ty> for $ty {
            fn allow(&mut self, value: $ty) {
                if *self != <$ty>::Forbidden {
                    *self = value;
                }
            }
            fn force_allow(&mut self, value: $ty) {
                *self = value;
            }
            fn forbid(&mut self) {
                *self = <$ty>::Forbidden;
            }
            fn soft_forbid(&mut self) {
                if *self == <$ty>::Unknown {
                    *self = <$ty>::Forbidden;
                }
            }
        })*
    }
}

macro_rules! add_allow_forbid_multiple {
    ($($nty:ty),*) => {
        $(add_allow_forbid!($nty);)*
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum FootAccessibility {
    Unknown,
    Forbidden,
    Allowed,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum CarAccessibility {
    Unknown,
    Forbidden,
    // @formatter:off
    Residential, // http://wiki.openstreetmap.org/wiki/Tag:highway%3Dresidential
    Tertiary,    // http://wiki.openstreetmap.org/wiki/Tag:highway%3Dtertiary
    Secondary,   // http://wiki.openstreetmap.org/wiki/Tag:highway%3Dsecondary
    Primary,     // http://wiki.http://wiki.openstreetmap.org/wiki/Tag:highway%3Dprimary
    Trunk,       // http://wiki.openstreetmap.org/wiki/Tag:highway%3Dtrunk
    Motorway,    // http://wiki.openstreetmap.org/wiki/Tag:highway%3Dmotorway
    // @formatter:on
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum BikeAccessibility {
    Unknown,
    Forbidden,
    // @formatter:off
    Allowed, // can be used by a bike, but the traffic might be shared with a car
    Lane,    // narrow lane dedicated for bikes, without physical separation from other traffic
    Busway,  // bikes are allowed on the bus lane
    Track,   // physically separated for any other traffic
    // @formatter:on
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum TrainAccessibility {
    Unknown,
    Forbidden,
    Allowed,
}

add_allow_forbid_multiple!(FootAccessibility, CarAccessibility, BikeAccessibility, TrainAccessibility);

// Edgeself contains what mode can use the edge in each direction
#[derive(Clone, Copy)]
pub struct EdgeProperties {
    pub foot: FootAccessibility,
    pub car_forward: CarAccessibility,
    pub car_backward: CarAccessibility,
    pub bike_forward: BikeAccessibility,
    pub bike_backward: BikeAccessibility,
    pub train: TrainAccessibility,
}

impl Default for EdgeProperties {
    fn default() -> EdgeProperties {
        EdgeProperties {
            foot: FootAccessibility::Unknown,
            car_forward: CarAccessibility::Unknown,
            car_backward: CarAccessibility::Unknown,
            bike_forward: BikeAccessibility::Unknown,
            bike_backward: BikeAccessibility::Unknown,
            train: TrainAccessibility::Unknown,
        }
    }
}

impl EdgeProperties {
    // Normalize fills UNKNOWN fields
    pub fn normalize(&mut self) {
        if self.car_backward == CarAccessibility::Unknown {
            self.car_backward = self.car_forward;
        }
        if self.bike_backward == BikeAccessibility::Unknown {
            self.bike_backward = self.bike_forward;
        }
        if self.car_forward == CarAccessibility::Unknown {
            self.car_forward = CarAccessibility::Forbidden;
        }
        if self.bike_forward == BikeAccessibility::Unknown {
            self.bike_forward = BikeAccessibility::Forbidden;
        }
        if self.car_backward == CarAccessibility::Unknown {
            self.car_backward = CarAccessibility::Forbidden;
        }
        if self.bike_backward == BikeAccessibility::Unknown {
            self.bike_backward = BikeAccessibility::Forbidden;
        }
        // if self.foot == FootAccessibility::Unknown {
        //     self.foot = FootAccessibility::Forbidden;
        // }
        if self.train == TrainAccessibility::Unknown {
            self.train = TrainAccessibility::Forbidden;
        }
    }

    // Accessible means that at least one mean of transportation can use it in one direction
    // but mark as not accessible if all are forbidden except FootAccessibility::Unknown
    pub fn accessible(self) -> bool {
        (self.bike_forward != BikeAccessibility::Forbidden
            || self.bike_backward != BikeAccessibility::Forbidden
            || self.car_forward != CarAccessibility::Forbidden
            || self.car_backward != CarAccessibility::Forbidden
            || self.foot != FootAccessibility::Forbidden
            || self.train != TrainAccessibility::Forbidden)
            && (self.foot != FootAccessibility::Unknown
            || self.car_forward != CarAccessibility::Forbidden
            || self.car_backward != CarAccessibility::Forbidden
            || self.bike_forward != BikeAccessibility::Forbidden
            || self.bike_backward != BikeAccessibility::Forbidden
            || self.train != TrainAccessibility::Forbidden
        )
    }

    pub fn update(&mut self, key_string: String, val_string: String) {
        let key = key_string.as_str();
        let val = val_string.as_str();
        self.update_with_str(key, val);
    }

    pub fn update_with_str(&mut self, key: &str, val: &str) {
        match key {
            "highway" => match val {
                "cycleway" => {
                    self.bike_forward.allow(BikeAccessibility::Track);
                    self.foot.allow(FootAccessibility::Allowed);
                }
                "path" | "footway" | "steps" | "pedestrian" => {
                    self.bike_forward.allow(BikeAccessibility::Allowed);
                    self.foot.allow(FootAccessibility::Allowed);
                }
                "primary" | "primary_link" => {
                    self.car_forward.allow(CarAccessibility::Primary);
                    self.foot.allow(FootAccessibility::Allowed);
                    self.bike_forward.allow(BikeAccessibility::Allowed);
                }
                "secondary" | "secondary_link" => {
                    self.car_forward.allow(CarAccessibility::Secondary);
                    self.foot.allow(FootAccessibility::Allowed);
                    self.bike_forward.allow(BikeAccessibility::Allowed);
                }
                "tertiary" | "tertiary_link" => {
                    self.car_forward.allow(CarAccessibility::Tertiary);
                    self.foot.allow(FootAccessibility::Allowed);
                    self.bike_forward.allow(BikeAccessibility::Allowed);
                }
                "unclassified" | "residential" | "living_street" | "road" | "service" | "track" => {
                    self.car_forward.allow(CarAccessibility::Residential);
                    self.foot.allow(FootAccessibility::Allowed);
                    self.bike_forward.allow(BikeAccessibility::Allowed);
                }
                "motorway" | "motorway_link" | "motorway_junction" => {
                    self.car_forward.allow(CarAccessibility::Motorway);
                    self.foot.soft_forbid();
                    self.bike_forward.soft_forbid();
                }
                "trunk" | "trunk_link" => {
                    self.car_forward.allow(CarAccessibility::Trunk);
                    self.foot.soft_forbid();
                    self.bike_forward.soft_forbid();
                }
                _ => {}
            },
            "motor_vehicle" => match val {
                "no" => {
                    self.car_forward.forbid();
                }
                _ => {}
            },
            "pedestrian" | "foot" => match val {
                "no" | "use_sidepath" => self.foot.forbid(),
                "yes" | "designated" | "official" => self.foot.force_allow(FootAccessibility::Allowed),
                "unknown" => self.foot.allow(FootAccessibility::Unknown),
                _ => self.foot.allow(FootAccessibility::Allowed),
            },
            "path" => if self.foot == FootAccessibility::Unknown {
                self.foot.allow(FootAccessibility::Allowed);
            },
            "sidewalk" => if self.foot == FootAccessibility::Unknown {
                self.foot.allow(FootAccessibility::Allowed);
            },
            "crossing" => if self.foot == FootAccessibility::Unknown {
                self.foot.allow(FootAccessibility::Allowed);
            },
            // http://wiki.openstreetmap.org/wiki/Cycleway
            // http://wiki.openstreetmap.org/wiki/Map_Features#Cycleway
            "cycleway" => match val {
                "track" => self.bike_forward.allow(BikeAccessibility::Track),
                "opposite_track" => self.bike_backward.allow(BikeAccessibility::Track),
                "opposite" => self.bike_backward.allow(BikeAccessibility::Allowed),
                "share_busway" => self.bike_forward.allow(BikeAccessibility::Busway),
                "lane_left" | "opposite_lane" => self.bike_backward.allow(BikeAccessibility::Lane),
                _ => self.bike_forward.allow(BikeAccessibility::Lane),
            },

            "bicycle" => match val {
                "no" | "false" | "use_sidepath" | "dismount" => self.bike_forward.forbid(),
                "yes" | "official" => self.bike_forward.force_allow(BikeAccessibility::Allowed),
                "designated" => self.bike_forward.force_allow(BikeAccessibility::Track),
                _ => self.bike_forward.allow(BikeAccessibility::Allowed),
            },
            "busway" => match val {
                "opposite_lane" | "opposite_track" => {
                    self.bike_backward.allow(BikeAccessibility::Busway)
                }
                _ => self.bike_forward.allow(BikeAccessibility::Busway),
            },
            "oneway" => match val {
                "yes" | "true" | "1" => {
                    self.car_backward.forbid();
                    if self.bike_backward == BikeAccessibility::Unknown {
                        self.bike_backward.forbid();
                    }
                }
                _ => {}
            },
            "junction" => {
                if val == "roundabout" {
                    self.car_backward.forbid();
                    if self.bike_backward == BikeAccessibility::Unknown {
                        self.bike_backward.forbid();
                    }
                }
            }
            "railway" => match val {
                "abandoned" | "disused" | "razed" => {
                    self.train.forbid();
                }
                _ => {
                    self.train.allow(TrainAccessibility::Allowed);
                    self.foot.soft_forbid();
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test_accessible() {
    let mut p = EdgeProperties::default();
    p.normalize();
    assert!(!p.accessible());

    p.foot = FootAccessibility::Allowed;
    assert!(p.accessible())
}

#[test]
fn test_normalize() {
    let mut p = EdgeProperties::default();
    p.bike_forward = BikeAccessibility::Lane;
    p.normalize();
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);
    p.bike_forward = BikeAccessibility::Allowed;
    p.normalize();
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);

    p.car_forward = CarAccessibility::Secondary;
    p.car_backward = CarAccessibility::Unknown;
    p.normalize();
    assert_eq!(CarAccessibility::Secondary, p.car_backward)
}

#[test]
fn test_update() {
    let mut p = EdgeProperties::default();
    p.update_with_str("highway", "secondary");
    assert_eq!(CarAccessibility::Secondary, p.car_forward);

    p.update_with_str("highway", "primary_link");
    assert_eq!(CarAccessibility::Primary, p.car_forward);

    p.update_with_str("highway", "motorway");
    assert_eq!(CarAccessibility::Motorway, p.car_forward);

    p.update_with_str("highway", "residential");
    assert_eq!(CarAccessibility::Residential, p.car_forward);

    p.update_with_str("highway", "tertiary");
    assert_eq!(CarAccessibility::Tertiary, p.car_forward);

    p.update_with_str("highway", "trunk");
    assert_eq!(CarAccessibility::Trunk, p.car_forward);

    p.update_with_str("highway", "cycleway");
    assert_eq!(BikeAccessibility::Track, p.bike_forward);
    assert_eq!(FootAccessibility::Allowed, p.foot);

    p.update_with_str("foot", "designated");
    assert_eq!(FootAccessibility::Allowed, p.foot);

    p.update_with_str("foot", "no");
    assert_eq!(FootAccessibility::Forbidden, p.foot);

    p.update_with_str("cycleway", "lane");
    assert_eq!(BikeAccessibility::Lane, p.bike_forward);

    p.update_with_str("cycleway", "track");
    assert_eq!(BikeAccessibility::Track, p.bike_forward);

    p.update_with_str("cycleway", "opposite_lane");
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);

    p.update_with_str("cycleway", "opposite_track");
    assert_eq!(BikeAccessibility::Track, p.bike_backward);

    p.update_with_str("cycleway", "opposite");
    assert_eq!(BikeAccessibility::Allowed, p.bike_backward);

    p.update_with_str("cycleway", "share_busway");
    assert_eq!(BikeAccessibility::Busway, p.bike_forward);

    p.update_with_str("cycleway", "lane_left");
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);

    p.update_with_str("bicycle", "yes");
    assert_eq!(BikeAccessibility::Allowed, p.bike_forward);

    p.update_with_str("bicycle", "no");
    assert_eq!(BikeAccessibility::Forbidden, p.bike_forward);

    p.update_with_str("busway", "yes");
    assert_eq!(BikeAccessibility::Busway, p.bike_forward);

    p.update_with_str("busway", "opposite_track");
    assert_eq!(BikeAccessibility::Busway, p.bike_backward);

    p.update_with_str("oneway", "yes");
    assert_eq!(CarAccessibility::Forbidden, p.car_backward);
    assert!(p.bike_backward != BikeAccessibility::Forbidden);

    p.bike_backward = BikeAccessibility::Unknown;
    p.update_with_str("oneway", "yes");
    assert_eq!(BikeAccessibility::Forbidden, p.bike_backward);

    p.update_with_str("junction", "roundabout");
    assert_eq!(CarAccessibility::Forbidden, p.car_backward);

    p.bike_backward = BikeAccessibility::Unknown;
    p.update_with_str("junction", "roundabout");
    assert_eq!(BikeAccessibility::Forbidden, p.bike_backward);
}
