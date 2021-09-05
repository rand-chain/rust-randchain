#!/bin/bash

cargo doc --no-deps\
	-p crypto\
	-p chain\
	-p db\
	-p import\
	-p message\
	-p miner\
	-p network\
	-p randchain\
	-p p2p\
	-p primitives\
	-p rpc\
	-p script\
	-p serialization\
	-p sync\
	-p test-data\
	-p verification
