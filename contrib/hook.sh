#!/bin/bash
set -xeo pipefail

www_dir=./public
archive_path=$www_dir/index.tar.xz
full_index_path=$www_dir/index.json
minimal_index_path=$www_dir/index.minimal.json

main() {
  asset_id=$1
  asset_path=$2

  echo "Registry in `pwd` updated, written asset $asset_id to $asset_path"

  # Commit to git and push
  if [ -d .git ]; then
    git add $asset_path
    # might fail with "nothing to commit" if the update didn't really change anything
    if git commit -S -m "Update asset $asset_id"; then
      git push
    fi
  fi

  # Make available in public www dir
  ln -s `realpath $asset_path` $www_dir/$asset_id.json

  # Update tar.xz archive
  tar cJf $archive_path ??/*.json

  # Maintain index.json with a full map of asset id -> asset data,
  # and index.minimal.json with a more concise representation
  json_full="$(cat $2)"
  json_minimal="$(cat $2 | jq -c '[.entity.domain,.ticker,.name,.precision]')"

  append_json_key $full_index_path $asset_id "$json_full"
  append_json_key $minimal_index_path $asset_id "$json_minimal"
}

# Assumes keys are only added and never updated (updating assets is currently not allowed by the api server)
append_json_key() {
  json_file=$1
  key=$2
  value=$3
  if [ ! -f $json_file ]; then
    echo -n '{' > $json_file
  else
    truncate -s-1 $json_file
    echo ',' >> $json_file
  fi
  echo -n '"'$key'":'"$value"'}' >> $json_file
}

#init_commit=`git rev-parse HEAD`
#rollback() {
#  git reset --hard $init_commit
#}

main "$@"
