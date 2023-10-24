#!/bin/bash

#Check if the .osm file exists
echo "Checking if latest.osm.pbf file exists..."
#echo "Checking if corse-latest.osm.pbf file exists..."
if [ ! -f "/data/latest.osm.pbf" ]; then
  #if [ ! -f "/data/corse-latest.osm.pbf" ]; then
  echo "latest.osm.pbf file does not exist."
  #  echo "corse-latest.osm.pbf file does not exist."
  echo "Downloading $OSM_FILE_URL file..."
  wget -S -nv -O /data/latest.osm.pbf $OSM_FILE_URL
else
  echo "latest.osm.pbf file exists."
fi

echo "Running osm4routing..."
cd /data || exit
/app/target/release/osm4routing latest.osm.pbf

#Exporting to S3_BUCKET
echo "Exporting to bucket $S3_BUCKET/osm4routing..."
#s4cmd put --verbose /data/latest.osm.pbf s3://$S3_BUCKET/osm4routing/latest.osm.pbf
#s4cmd put /data/edges.csv s3://$S3_BUCKET/osm4routing/edges.csv
#s4cmd put /data/nodes.csv s3://$S3_BUCKET/osm4routing/nodes.csv
for file in /data/*.csv; do
  filename=$(basename $file)
  echo "Exporting $file to bucket $S3_BUCKET/osm4routing/$filename..."
  s4cmd put $file s3://$S3_BUCKET/osm4routing/$filename
done
echo "Done!"
