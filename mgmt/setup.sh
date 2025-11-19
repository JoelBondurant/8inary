#!/usr/bin/env bash

sudo useradd -m -s /bin/bash mgmt
sudo usermod -aG sudo mgmt

echo "mgmt ALL=(ALL) NOPASSWD: ALL" | sudo tee /etc/sudoers.d/91-mgmt-nopasswd

