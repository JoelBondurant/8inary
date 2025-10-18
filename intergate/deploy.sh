printf "intergate deployment started...  %s\n" "$(date -u -Iseconds)"
ssh ubuntu@dfw0 useradd --system --shell /sbin/nologin -m intergate 2> /dev/null
scp intergate.service ubuntu@dfw0:/home/ubuntu/intergate.service
ssh ubuntu@dfw0 sudo mv -f /home/ubuntu/intergate.service /etc/systemd/system/intergate.service
ssh ubuntu@dfw0 sudo systemctl daemon-reload
ssh ubuntu@dfw0 sudo systemctl enable intergate
scp target/release/intergate ubuntu@dfw0:/home/ubuntu/intergate.deploy
ssh ubuntu@dfw0 sudo mv -f /home/ubuntu/intergate.deploy /home/intergate/intergate.deploy
ssh ubuntu@dfw0 systemctl status intergate
ssh ubuntu@dfw0 systemctl stop intergate
ssh ubuntu@dfw0 systemctl status intergate
ssh ubuntu@dfw0 sudo mv -f /home/intergate/intergate.deploy /home/intergate/intergate
ssh ubuntu@dfw0 sudo setcap 'cap_net_bind_service=+ep' /home/intergate/intergate
ssh ubuntu@dfw0 systemctl start intergate
ssh ubuntu@dfw0 systemctl status intergate
printf "intergate deployment finished.  %s\n" "$(date -u -Iseconds)"

