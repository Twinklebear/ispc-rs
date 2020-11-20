#!/bin/bash

# build the examples
cd examples
for d in `ls ./`; do
	cd $d
	pwd
    if [[ "$d" == "simple" ]]; then
        cargo build --features ispc
    fi

	cargo build
	if [[ "$?" != "0" ]]; then
		exit 1
	fi
	cd ../
done

