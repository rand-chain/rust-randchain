#!/bin/bash

AWS_REGIONS=(
    us-east-1
    us-west-1
    ap-south-1
    ap-northeast-2
    ap-southeast-2
    ap-northeast-1
    ca-central-1
    eu-west-1
    sa-east-1
    us-west-2
    us-east-2
    ap-southeast-1
    eu-west-2
    eu-central-1
)

for each in ${AWS_REGIONS}
do
aws ec2 import-key-pair --key-name randchain --public-key-material file://~/.ssh/randchain.pub --region $each
done


