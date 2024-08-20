use anyhow::Result;
use geo::{EuclideanDistance, EuclideanLength, LineInterpolatePoint, LineString};

use crate::graph::{Graph, Mode, PathStep};

impl Graph {
    /// Given an input LineString (in Mercator), try to snap/map-match it to a given Mode's graph
    pub fn snap_route(
        &self,
        input: &LineString,
        mode: Mode,
    ) -> Result<(Vec<PathStep>, LineString)> {
        // TODO Simple start: just match the endpoints and find the optimal route, according to
        // that mode's graph.

        let start = self.snap_to_road(*input.coords().next().unwrap(), mode);
        let end = self.snap_to_road(*input.coords().last().unwrap(), mode);
        let steps = self.router[mode].route_steps(self, start, end)?;

        // TODO Repeats work until PathStep -> LineString has a proper API
        let output = self.router[mode].route_linestring(self, start, end)?;

        score_similarity(input, &output);

        Ok((steps, output))
    }
}

fn score_similarity(ls1: &LineString, ls2: &LineString) {
    // Just check length
    let len1 = ls1.euclidean_length();
    let len2 = ls2.euclidean_length();
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
        let pt1 = ls1.line_interpolate_point(pct).unwrap();
        let pt2 = ls2.line_interpolate_point(pct).unwrap();
        dist_between_equiv_pts += pt1.euclidean_distance(&pt2);
    }

    info!("snap_route scores: {len_pct} ratio (1 is perfect), {dist_between_equiv_pts} sum distance between equivalent points");
}
