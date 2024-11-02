use anyhow::Result;
use geo::{
    Closest, ClosestPoint, Distance, Euclidean, Length, LineInterpolatePoint, LineLocatePoint,
    LineString,
};

use crate::{Graph, IntersectionID, PathStep, ProfileID, Route};

impl Graph {
    /// Given an input LineString (in Mercator), try to snap/map-match it to a given profile's graph
    pub fn snap_route(&self, input: &LineString, profile: ProfileID) -> Result<Route> {
        if false {
            self.snap_by_endpoints(input, profile)
        } else {
            self.snap_greedy(input, profile)
        }
    }

    fn snap_by_endpoints(&self, input: &LineString, profile: ProfileID) -> Result<Route> {
        // Simple start: just match the endpoints and find the optimal route, according to
        // that profile's graph.
        let start = self.snap_to_road(*input.coords().next().unwrap(), profile);
        let end = self.snap_to_road(*input.coords().last().unwrap(), profile);
        let route = self.routers[profile.0].route(self, start, end)?;

        // TODO Detect/handle zero-length output here

        Ok(route)
    }

    fn snap_greedy(&self, input: &LineString, profile: ProfileID) -> Result<Route> {
        let start = self.snap_to_road(*input.coords().next().unwrap(), profile);
        let end = self.snap_to_road(*input.coords().last().unwrap(), profile);

        let mut current = start.intersection;
        let mut fraction_along = 0.0;
        let mut steps = Vec::new();

        while current != end.intersection {
            match self
                .next_steps(current)
                .filter_map(|(i, step)| {
                    // Find the closest point on the input linestring to this possible next
                    // intersection
                    match input.closest_point(&self.intersections[i.0].point) {
                        Closest::Intersection(pt) | Closest::SinglePoint(pt) => {
                            // How far along on the input linestring is it? If we'd move backwards,
                            // skip it
                            let new_fraction_along = input.line_locate_point(&pt)?;
                            if new_fraction_along > fraction_along {
                                let dist = (100.0
                                    * Euclidean::distance(pt, self.intersections[i.0].point))
                                    as usize;
                                Some((i, step, dist, new_fraction_along))
                            } else {
                                None
                            }
                        }
                        Closest::Indeterminate => None,
                    }
                })
                // TODO Maybe also use new_fraction_along to judge the best next step
                .min_by_key(|(_, _, dist, _)| *dist)
            {
                Some((i, step, _, new_fraction_along)) => {
                    fraction_along = new_fraction_along;
                    steps.push(step);
                    current = i;
                }
                None => bail!("Got stuck at {}", self.intersections[current.0].node),
            }
        }

        Ok(Route { steps, start, end })
    }

    // TODO Ignores profile and direction
    fn next_steps(
        &self,
        i: IntersectionID,
    ) -> impl Iterator<Item = (IntersectionID, PathStep)> + '_ {
        self.intersections[i.0].roads.iter().flat_map(move |r| {
            let road = &self.roads[r.0];
            if road.src_i != i {
                Some((
                    road.src_i,
                    PathStep::Road {
                        road: *r,
                        forwards: false,
                    },
                ))
            } else if road.dst_i != i {
                Some((
                    road.dst_i,
                    PathStep::Road {
                        road: *r,
                        forwards: true,
                    },
                ))
            } else {
                // A loop on i
                None
            }
        })
    }
}

// TODO Reconsider exposing
pub fn score_similarity(ls1: &LineString, ls2: &LineString) -> Option<(f64, f64)> {
    // Just check length
    let len1 = ls1.length::<Euclidean>();
    let len2 = ls2.length::<Euclidean>();
    let len_pct = if len1 < len2 {
        len2 / len1
    } else {
        len1 / len2
    };

    // Walk along each and take the distance between (hopefully equivalent) points. This only
    // really makes sense if the lengths are the same. Not sure how to scale this.
    let mut dist_between_equiv_pts = 0.0;
    for pct in 0..=100 {
        let pct = (pct as f64) / 100.0;
        // TODO Where do we have zero-length lines?
        let pt1 = ls1.line_interpolate_point(pct)?;
        let pt2 = ls2.line_interpolate_point(pct)?;

        dist_between_equiv_pts += Euclidean::distance(pt1, pt2);
    }

    Some((len_pct, dist_between_equiv_pts))
}
