#!/bin/bash

scp -i "~/.ssh/randchain.pem" $1 ec2-user@$2:/home/ec2-user

