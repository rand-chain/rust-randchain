#!/bin/bash

yum -y install cmake gcc m4 gmp gmp-devel mpfr mpfr-devel libmpc libmpc-devel dstat
wget -O /home/ec2-user/randchaind https://randchain-dev.s3-us-west-1.amazonaws.com/randchaind