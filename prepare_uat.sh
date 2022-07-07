#!/bin/sh

volumes=(db mongo cache)

for i in ${volumes[@]} ; do
	docker container run --rm -it \
           -v ffb_$i:/from \
           -v ffb_uat_$i:/to \
           alpine ash -c "cd /from ; cp -av . /to"
done
