var searchIndex = new Map(JSON.parse('[\
["graph",{"t":"PPPPPGIPPPFFGFFGPPGFFPFFFFPONNNNNNNNNNNNNNNNNNNNNNNNNNNNNNONNNNNNNNNNNNNNNNNNNNNONNNNONNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNONONNNNNNNNNNNNNNNNONNNNNNNNNNNNNNNNNNONNOONNNNNNNNNNNNNNOONNNNNNNNNNNNNNNNNNNNNNNNNNNNNONONOONNNOOOONNNNONNNOOONOONNNNNNNNNNCNNOONOOONNNNNNNNONNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNOOOOOOH","n":["Backwards","Bicycle","Both","Car","Dir","Direction","EdgeLocation","Foot","Forwards","Geomedea","Graph","GtfsModel","GtfsSource","Intersection","IntersectionID","Mode","None","None","PathStep","Position","Road","Road","RoadID","Route","Router","Timer","Transit","access","allows_backwards","allows_forwards","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","boundary_polygon","by_distance","clone","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","clone_into","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","clone_to_uninit","closest_road","cmp","cmp","compare","compare","data","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deref_mut","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","deserialize","done","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","drop","dst_i","empty","end","eq","eq","eq","eq","equivalent","equivalent","equivalent","equivalent","equivalent","equivalent","find_edge","fmt","fmt","fmt","fmt","fmt","fraction_along","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from_geomedea","from_usize","get_costs","get_inverted_boundary","gtfs","hash","hash","id","id","init","init","init","init","init","init","init","init","init","init","init","init","init","init","intersection","intersections","into","into","into","into","into","into","into","into","into","into","into","into","into","into","into_usize","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","is_within","length_meters","linestring","linestring","log","max_speed","mercator","new","new","new","node","node1","node2","osm_tags","parse","parse","partial_cmp","partial_cmp","point","pop","push","render_debug","road","roads","roads","route","router","routes","serialize","serialize","serialize","serialize","serialize","serialize","serialize","serialize","serialize","setup_gtfs","snap","snap_route","snap_to_road","src_i","start","step","steps","stops","stops","to_geomedea","to_gj","to_owned","to_owned","to_owned","to_owned","to_owned","transit_route_gj","trips","trips_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","way","forwards","road","stop1","stop2","trip","score_similarity"],"q":[[0,"graph"],[325,"graph::PathStep"],[330,"graph::snap"],[331,"alloc::vec"],[332,"graph::route"],[333,"core::cmp"],[334,"graph::gtfs"],[335,"core::result"],[336,"serde::de"],[337,"graph::timer"],[338,"core::fmt"],[339,"utils::mercator"],[340,"anyhow"],[341,"chrono::naive::time"],[342,"core::time"],[343,"std::collections::hash::map"],[344,"alloc::string"],[345,"core::hash"],[346,"geo_types::geometry::line_string"],[347,"core::convert"],[348,"js_sys"],[349,"core::option"],[350,"utils::osm2graph"],[351,"core::ops::function"],[352,"geojson::feature_collection"],[353,"serde::ser"],[354,"geo_types::geometry::coord"],[355,"geojson"],[356,"core::any"]],"i":[10,3,10,3,42,0,0,3,10,42,0,0,0,0,0,0,42,10,0,0,0,48,0,0,0,0,48,1,1,1,31,20,42,48,15,7,18,8,9,10,3,1,19,11,31,20,42,48,15,7,18,8,9,10,3,1,19,11,18,7,8,9,10,3,11,8,9,10,3,11,8,8,9,9,10,10,3,3,11,11,18,8,9,8,9,49,31,20,42,48,15,7,18,8,9,10,3,1,19,11,31,20,42,48,15,7,18,8,9,10,3,1,19,11,15,7,18,8,9,10,3,1,19,20,31,20,42,48,15,7,18,8,9,10,3,1,19,11,1,15,31,8,9,10,11,8,8,8,9,9,9,18,8,9,10,3,11,11,31,20,42,48,15,7,18,8,9,10,3,1,19,11,15,3,18,18,18,8,9,1,19,31,20,42,48,15,7,18,8,9,10,3,1,19,11,11,18,31,20,42,48,15,7,18,8,9,10,3,1,19,11,3,31,20,42,48,15,7,18,8,9,10,3,1,19,11,1,31,1,20,1,18,20,7,18,19,1,1,1,15,3,8,9,19,20,20,18,11,18,19,7,18,15,15,7,18,8,9,10,3,1,19,18,0,18,18,1,31,20,31,15,1,15,1,8,9,10,3,11,18,15,15,31,20,42,48,15,7,18,8,9,10,3,1,19,11,31,20,42,48,15,7,18,8,9,10,3,1,19,11,31,20,42,48,15,7,18,8,9,10,3,1,19,11,1,50,50,51,51,51,0],"f":"````````````````````````````{{{d{b}}f}h}0{{{d{c}}}{{d{e}}}{}{}}0000000000000{{{d{jc}}}{{d{je}}}{}{}}0000000000000`{{{d{{l{b}}}}}n}{{{d{A`}}}A`}{{{d{Ab}}}Ab}{{{d{Ad}}}Ad}{{{d{f}}}f}{{{d{Af}}}Af}{{{d{c}}{d{je}}}Ah{}{}}0000{{{d{c}}}Ah{}}000000000`{{{d{A`}}{d{A`}}}Aj}{{{d{Ab}}{d{Ab}}}Aj}{{{d{c}}{d{e}}}Aj{}{}}0`{Al{{d{c}}}{}}0000000000000{Al{{d{jc}}}{}}0000000000000{c{{B`{An}}}Bb}{c{{B`{n}}}Bb}{c{{B`{Bd}}}Bb}{c{{B`{A`}}}Bb}{c{{B`{Ab}}}Bb}{c{{B`{Ad}}}Bb}{c{{B`{f}}}Bb}{c{{B`{b}}}Bb}{c{{B`{Bf}}}Bb}{BhAh}{AlAh}0000000000000`{{}An}`{{{d{A`}}{d{A`}}}h}{{{d{Ab}}{d{Ab}}}h}{{{d{Ad}}{d{Ad}}}h}{{{d{Af}}{d{Af}}}h}{{{d{c}}{d{e}}}h{}{}}00000{{{d{Bd}}AbAb}{{d{b}}}}{{{d{A`}}{d{jBj}}}Bl}{{{d{Ab}}{d{jBj}}}Bl}{{{d{Ad}}{d{jBj}}}Bl}{{{d{f}}{d{jBj}}}Bl}{{{d{Af}}{d{jBj}}}Bl}`{cc{}}0000000000000{{{d{Bn}}{d{C`}}}{{Cb{An}}}}{Alf}{{{d{Bd}}{l{Ab}}fhCdCd}{{Ch{A`Cf}}}}{{{d{Bd}}}{{Cb{Cj}}}}`{{{d{A`}}{d{jc}}}AhCl}{{{d{Ab}}{d{jc}}}AhCl}``{{}Al}0000000000000``{ce{}{}}0000000000000{fAl}{{{d{c}}{d{e}}}h{}{}}0000000000000`{{{d{Cn}}{d{Bd}}}D`}`{{{d{Bh}}c}Ah{{Db{Cj}}}}``{{c{Df{Dd}}}Bh{{Db{Cj}}}}{{{d{{l{b}}}}f}n}{{{d{{Dj{Dh}}}}{d{jc}}e{d{jBh}}}{{Cb{Bd}}}Dl{{Dn{{d{j{l{b}}}}}}}}````{{{d{Bn}}{Df{{d{C`}}}}}{{Cb{An}}}}{{{d{Bn}}}{{Cb{f}}}}{{{d{A`}}{d{A`}}}{{Df{Aj}}}}{{{d{Ab}}{d{Ab}}}{{Df{Aj}}}}`{{{d{jBh}}}Ah}{{{d{jBh}}c}Ah{{Db{Cj}}}}{{{d{Bd}}}E`}```{{{d{n}}{d{Bd}}AfAf}{{Cb{Cn}}}}``{{{d{An}}c}B`Eb}{{{d{n}}c}B`Eb}{{{d{Bd}}c}B`Eb}{{{d{A`}}c}B`Eb}{{{d{Ab}}c}B`Eb}{{{d{Ad}}c}B`Eb}{{{d{f}}c}B`Eb}{{{d{b}}c}B`Eb}{{{d{Bf}}c}B`Eb}{{{d{jBd}}Ed{d{jBh}}}{{Cb{Ah}}}}`{{{d{Bd}}{d{D`}}f}{{Cb{Cn}}}}{{{d{Bd}}Eff}Af}``>```{{{d{An}}{d{Bn}}}{{Cb{Ah}}}}{{{d{b}}{d{C`}}}Eh}{{{d{c}}}e{}{}}0000{{{d{Bd}}AfAfhhCdBh}{{Cb{Cj}}}}``{c{{B`{e}}}{}{}}000000000000000000000000000{{{d{c}}}Ej{}}0000000000000``````{{{d{D`}}{d{D`}}}{{Df{{En{ElEl}}}}}}","D":"Gn","p":[[5,"Road",0],[1,"reference"],[6,"Mode",0],[1,"bool"],[0,"mut"],[5,"Vec",331],[5,"Router",0,332],[5,"RoadID",0],[5,"IntersectionID",0],[6,"Direction",0],[5,"Position",0],[1,"unit"],[6,"Ordering",333],[1,"usize"],[5,"GtfsModel",0,334],[6,"Result",335],[10,"Deserializer",336],[5,"Graph",0],[5,"Intersection",0],[5,"Timer",0,337],[5,"Formatter",338],[8,"Result",338],[1,"str"],[5,"Mercator",339],[8,"Result",340],[5,"NaiveTime",341],[5,"Duration",342],[5,"HashMap",343],[5,"String",344],[10,"Hasher",345],[5,"Route",0,332],[5,"LineString",346],[10,"Into",347],[5,"Function",348],[6,"Option",349],[1,"u8"],[1,"slice"],[10,"OsmReader",350],[10,"FnOnce",351],[5,"FeatureCollection",352],[10,"Serializer",353],[6,"GtfsSource",0],[5,"Coord",354],[5,"Feature",355],[5,"TypeId",356],[1,"f64"],[1,"tuple"],[6,"PathStep",0],[8,"EdgeLocation",0],[15,"Road",325],[15,"Transit",325]],"r":[[11,334],[23,332],[24,332],[25,337]],"b":[],"c":"OjAAAAAAAAA=","e":"OzAAAAEAAAwBGQAAAAUABwADAAwAAQAPAAAAEQABABYAAQAaAAEAHwAbAD0AGABXAEAAmQAFAK0AAQCxABQA1AAPAOUAAQDpAAAA7AADAPIAAgD4AAIA/AAKAAgBAAALAQEADgEBABEBCAAbATAA"}]\
]'));
if (typeof exports !== 'undefined') exports.searchIndex = searchIndex;
else if (window.initSearch) window.initSearch(searchIndex);
