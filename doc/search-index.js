var searchIndex = new Map(JSON.parse('[\
["graph",{"t":"PPPGPPFFGFFPPGFFFPFFFFPONNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNONNNNNNNNNNNNNNNNNNNNONNNNNNONNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNONONNNNNNNNNNNNNNNNNNNNNNNNONNNNNNNNNNNNNNNNONNNOONNNNNNNNNNNNNNOONNNNNNNNNNNNNNNNNNNNNNNNNNNNONONONNNOOOONNNNONONNOOONOONNNNNNNNNCNNOONOOONNNNNNNONNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNOOOOOOOH","n":["Backwards","Both","Dir","Direction","Forwards","Geomedea","Graph","GtfsModel","GtfsSource","Intersection","IntersectionID","None","None","PathStep","Position","ProfileID","Road","Road","RoadID","Route","Router","Timer","Transit","access","add_profile","allows_backwards","allows_forwards","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","boundary_polygon","clone","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","clone_into","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","closest_road","cmp","cmp","cmp","compare","compare","compare","cost","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","done","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","dst_i","empty","end","eq","eq","eq","eq","eq","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","find_edge","fmt","fmt","fmt","fmt","fmt","fmt","fraction_along","from","from","from","from","from","from","from","from","from","from","from","from","from","from","get_costs","get_inverted_boundary","gtfs","hash","hash","hash","id","id","init","init","init","init","init","init","init","init","init","init","init","init","init","init","intersection","intersections","into","into","into","into","into","into","into","into","into","into","into","into","into","into","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","length_meters","linestring","linestring","log","mercator","new","new","new","node","node1","node2","osm_tags","parse","partial_cmp","partial_cmp","partial_cmp","point","pop","profile_names","push","render_debug","road","roads","roads","route","routers","routes","serialize","serialize","serialize","serialize","serialize","serialize","serialize","serialize","serialize","snap","snap_route","snap_to_road","src_i","start","step","steps","stops","stops","to_gj","to_owned","to_owned","to_owned","to_owned","to_owned","transit_route_gj","trips","trips_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","walking_profile_for_transit","way","forwards","road","stop1","stop2","trip","score_similarity"],"q":[[0,"graph"],[328,"graph::PathStep"],[333,"graph::snap"],[334,"alloc::string"],[335,"core::ops::function"],[336,"alloc::boxed"],[337,"core::cmp"],[338,"graph::gtfs"],[339,"core::result"],[340,"serde::de"],[341,"graph::route"],[342,"graph::timer"],[343,"core::option"],[344,"core::fmt"],[345,"alloc::vec"],[346,"chrono::naive::time"],[347,"core::time"],[348,"std::collections::hash::map"],[349,"anyhow"],[350,"core::hash"],[351,"geo_types::geometry::line_string"],[352,"core::convert"],[353,"js_sys"],[354,"utils::osm2graph"],[355,"utils::mercator"],[356,"geojson::feature_collection"],[357,"serde::ser"],[358,"geo_types::geometry::coord"],[359,"geojson"],[360,"core::any"]],"i":[12,12,49,0,12,49,0,0,0,0,0,49,12,0,0,0,0,26,0,0,0,0,26,8,2,8,8,33,22,49,17,20,2,10,11,7,12,8,21,13,26,33,22,49,17,20,2,10,11,7,12,8,21,13,26,2,10,11,7,12,13,10,11,7,12,13,10,10,11,11,7,7,12,12,13,13,20,10,11,7,10,11,7,8,33,22,49,17,20,2,10,11,7,12,8,21,13,26,33,22,49,17,20,2,10,11,7,12,8,21,13,26,17,20,2,10,11,7,12,8,21,22,33,22,49,17,20,2,10,11,7,12,8,21,13,26,8,17,33,10,11,7,12,13,10,10,10,10,11,11,11,11,7,7,7,7,2,10,11,7,12,13,26,13,33,22,49,17,20,2,10,11,7,12,8,21,13,26,2,2,2,10,11,7,8,21,33,22,49,17,20,2,10,11,7,12,8,21,13,26,13,2,33,22,49,17,20,2,10,11,7,12,8,21,13,26,33,22,49,17,20,2,10,11,7,12,8,21,13,26,8,33,8,22,2,22,20,2,21,8,8,8,17,10,11,7,21,22,2,22,2,13,2,21,20,2,17,17,20,2,10,11,7,12,8,21,0,2,2,8,33,22,33,17,8,8,10,11,7,12,13,2,17,17,33,22,49,17,20,2,10,11,7,12,8,21,13,26,33,22,49,17,20,2,10,11,7,12,8,21,13,26,33,22,49,17,20,2,10,11,7,12,8,21,13,26,2,8,50,50,51,51,51,0],"f":"````````````````````````{{{f{bd}}h{l{j}}}n}{{{f{A`}}n}Ab}0{{{f{c}}}{{f{e}}}{}{}}0000000000000{{{f{bc}}}{{f{be}}}{}{}}0000000000000`{{{f{Ad}}}Ad}{{{f{Af}}}Af}{{{f{n}}}n}{{{f{Ah}}}Ah}{{{f{Aj}}}Aj}{{{f{c}}{f{be}}}Al{}{}}0000{{{f{c}}}Al{}}000000000`{{{f{Ad}}{f{Ad}}}An}{{{f{Af}}{f{Af}}}An}{{{f{n}}{f{n}}}An}{{{f{c}}{f{e}}}An{}{}}00`{B`{{f{c}}}{}}0000000000000{B`{{f{bc}}}{}}0000000000000{c{{Bd{Bb}}}Bf}{c{{Bd{Bh}}}Bf}{c{{Bd{d}}}Bf}{c{{Bd{Ad}}}Bf}{c{{Bd{Af}}}Bf}{c{{Bd{n}}}Bf}{c{{Bd{Ah}}}Bf}{c{{Bd{A`}}}Bf}{c{{Bd{Bj}}}Bf}{BlAl}{B`Al}0000000000000`{{}Bb}`{{{f{Ad}}{f{Ad}}}Ab}{{{f{Af}}{f{Af}}}Ab}{{{f{n}}{f{n}}}Ab}{{{f{Ah}}{f{Ah}}}Ab}{{{f{Aj}}{f{Aj}}}Ab}{{{f{c}}{f{e}}}Ab{}{}}00000000000{{{f{d}}AfAf}{{Bn{{f{A`}}}}}}{{{f{Ad}}{f{bC`}}}Cb}{{{f{Af}}{f{bC`}}}Cb}{{{f{n}}{f{bC`}}}Cb}{{{f{Ah}}{f{bC`}}}Cb}{{{f{Aj}}{f{bC`}}}Cb}{{{f{Cd}}{f{bC`}}}Cb}`{cc{}}0000000000000{{{f{d}}{Cf{Af}}nAbChCh}{{Cl{AdCj}}}}{{{f{d}}}{{Cn{h}}}}`{{{f{Ad}}{f{bc}}}AlD`}{{{f{Af}}{f{bc}}}AlD`}{{{f{n}}{f{bc}}}AlD`}``{{}B`}0000000000000``{ce{}{}}0000000000000??????????????`{{{f{Db}}{f{d}}}Dd}`{{{f{Bl}}c}Al{{Df{h}}}}`{{c{Bn{Dh}}}Bl{{Df{h}}}}{{{f{{Cf{A`}}}}n}Bh}{{{f{{Dl{Dj}}}}{f{bc}}{Cf{{Dn{h{l{j}}}}}}{f{bBl}}}{{Cn{d}}}E`}````{{{f{Eb}}{Bn{{f{Ed}}}}}{{Cn{Bb}}}}{{{f{Ad}}{f{Ad}}}{{Bn{An}}}}{{{f{Af}}{f{Af}}}{{Bn{An}}}}{{{f{n}}{f{n}}}{{Bn{An}}}}`{{{f{bBl}}}Al}`{{{f{bBl}}c}Al{{Df{h}}}}{{{f{d}}}Ef}```{{{f{Bh}}{f{d}}AjAj}{{Cn{Db}}}}``{{{f{Bb}}c}BdEh}{{{f{Bh}}c}BdEh}{{{f{d}}c}BdEh}{{{f{Ad}}c}BdEh}{{{f{Af}}c}BdEh}{{{f{n}}c}BdEh}{{{f{Ah}}c}BdEh}{{{f{A`}}c}BdEh}{{{f{Bj}}c}BdEh}`{{{f{d}}{f{Dd}}n}{{Cn{Db}}}}{{{f{d}}Ejn}Aj}``=```{{{f{A`}}{f{d}}}El}{{{f{c}}}e{}{}}0000{{{f{d}}AjAjAbAbChBl}{{Cn{h}}}}``{c{{Bd{e}}}{}{}}000000000000000000000000000{{{f{c}}}En{}}0000000000000```````{{{f{Dd}}{f{Dd}}}{{Bn{{Dn{F`F`}}}}}}","D":"Gf","p":[[0,"mut"],[5,"Graph",0],[1,"reference"],[5,"String",334],[10,"Fn",335],[5,"Box",336],[5,"ProfileID",0],[5,"Road",0],[1,"bool"],[5,"RoadID",0],[5,"IntersectionID",0],[6,"Direction",0],[5,"Position",0],[1,"unit"],[6,"Ordering",337],[1,"usize"],[5,"GtfsModel",0,338],[6,"Result",339],[10,"Deserializer",340],[5,"Router",0,341],[5,"Intersection",0],[5,"Timer",0,342],[6,"Option",343],[5,"Formatter",344],[8,"Result",344],[6,"PathStep",0],[5,"Vec",345],[5,"NaiveTime",346],[5,"Duration",347],[5,"HashMap",348],[8,"Result",349],[10,"Hasher",350],[5,"Route",0,341],[5,"LineString",351],[10,"Into",352],[5,"Function",353],[1,"u8"],[1,"slice"],[1,"tuple"],[10,"OsmReader",354],[1,"str"],[5,"Mercator",355],[5,"FeatureCollection",356],[10,"Serializer",357],[5,"Coord",358],[5,"Feature",359],[5,"TypeId",360],[1,"f64"],[6,"GtfsSource",0],[15,"Road",328],[15,"Transit",328]],"r":[[7,338],[19,341],[20,341],[21,342]],"b":[],"c":"OjAAAAAAAAA=","e":"OzAAAAEAABMBGQAAAAMABQABAAgAAQALAAIAEAAAABIAAQAWAAEAGQAAABwAGwA5ABoAVQBHAJ4ABgC1ABUA2QAOAOkAAQDsAAAA7wADAPQAAwD5AAAA/AACAAEBCgAOAQEAEQEBABQBBwAdATEA"}]\
]'));
if (typeof exports !== 'undefined') exports.searchIndex = searchIndex;
else if (window.initSearch) window.initSearch(searchIndex);
