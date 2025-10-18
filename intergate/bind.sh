printf "binding to port 443 started...  %s\n" "$(date -u -Iseconds)"
sudo setcap 'cap_net_bind_service=+ep' target/release/intergate
printf "binding to port 443 finished.  %s\n" "$(date -u -Iseconds)"

