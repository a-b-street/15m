searchState.loadedDescShard("graph", 0, "How can a <code>Road</code> be crossed by a particular profile?\nA study area imported from OpenStreetMap.\nAn intersection between one or more roads. This might …\nA single step along a route\nA position along a road, along with the closest …\nRepresents an edge going between exactly two <code>Intersection</code>s.\nA route between two positions.\nManages routing queries for one profile. This structure …\nPer profile, what direction is this road traversable?\nCan this profile cross this road in the backwards …\nCan this profile cross this road in the forwards direction?\nA polygon covering the study area.\nWhat’s the cost of crossing this road? If there’s no …\nFind the Road going from <code>i1</code> to <code>i2</code> or vice versa\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nFrom a list of start intersections, floods out the graph …\nReturn a polygon covering the world, minus a hole for the …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nRenders a route as a linestring (in Mercator), with …\n<code>Graph</code> stores all geometry in a Mercator projection for the …\nCreates a router for a profile. This is slow to calculate, …\nConstructs a graph from OpenStreetMap data.\nTakes a path to a GTFS directory. If no Mercator is …\nStop a nested step\nStart a new step with nested steps following it\nReturns GeoJSON with roads and stops\nCalculates a route between two positions.\nPer profile\nGiven an input LineString (in Mercator), try to …\nGiven a point (in Mercator) and profile, snap to a …\nStart a new step, with no nesting\nThe bus stops associated with this road\nStarting from a stop at some time, find all the next trips …")