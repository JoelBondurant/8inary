printf "intergate starting...  %s\n" "$(date -u -Iseconds)"
for i in {1..3}; do ./bind.sh && break || sleep 2; done
#cargo watch -s "./bind.sh" -x run
sleep 1
./target/release/intergate

