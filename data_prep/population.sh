#!/bin/bash
# This script prepares a consolidated FGB file with total population count in
# the most granular zone for different countries, using
# https://github.com/Urban-Analytics-Technology-Platform/popgetter. This script
# / process might live elsewhere at some point.

set -e
set -x

# You need to build https://github.com/Urban-Analytics-Technology-Platform/popgetter-cli
POPGETTER=/home/dabreegster/Downloads/popgetter-cli/target/release/popgetter

# The metrics and coordinate systems were figured out manually. In the future,
# this'll be easier with a popgetter web UI.

# England
$POPGETTER data \
        --force-run \
        --output-format geojson \
        --output-file england_raw.geojson \
        --geometry-level oa \
        --id 1355501cf6f3b1fa8cf6a100c98f330d51a3382ed2111fef0d2fff446608a428
mapshaper england_raw.geojson \
        -rename-fields population='Residence type: Total; measures: Value' \
        -each 'delete GEO_ID' \
        -proj init=EPSG:27700 crs=wgs84 \
        -o england.geojson

# Belgium
$POPGETTER data \
        --force-run \
        --output-format geojson \
        --output-file belgium_raw.geojson \
        --geometry-level statistical_sector \
        --id fcf09809889c1d9715bff5f825b0c6ed4d9286f2e2b4948839accc29c15e98c5
mapshaper belgium_raw.geojson \
        -rename-fields population='TOTAL' \
        -each 'delete GEO_ID' \
        -proj init=EPSG:3812 crs=wgs84 \
        -o belgium.geojson

# USA
# TODO This might not be the right variable, and it's only at block_group
$POPGETTER data \
        --force-run \
        --output-format geojson \
        --output-file usa_raw.geojson \
        --geometry-level block_group \
        --id d23e348af6ab03265b4f258178edc6b509651095f81b965c1a62396fe463d0f6
mapshaper usa_raw.geojson \
        -rename-fields population='B01001_E001' \
        -each 'delete GEO_ID' \
        -o usa.geojson

# Scotland
# TODO

# Northern Ireland
# TODO

# Merge files. You need to build https://github.com/acteng/will-it-fit/tree/main/data_prep/merge_files
MERGER=/home/dabreegster/will-it-fit/data_prep/merge_files/target/release/merge_files
$MERGER england.geojson belgium.geojson usa.geojson
# Hosting: mv out.fgb ~/cloudflare_sync/population.fgb, sync it
