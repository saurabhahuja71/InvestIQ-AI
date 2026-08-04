#!/usr/bin/env bash
# Permanently allow InvestIQ API (TCP 8080) through the host firewall.
set -euo pipefail
PORT="${PORT:-8080}"
if command -v firewall-cmd >/dev/null 2>&1 && firewall-cmd --state 2>/dev/null | grep -qi running; then
  sudo firewall-cmd --permanent --add-port="${PORT}/tcp"
  # Also ensure the active zone allows it
  ZONE="$(firewall-cmd --get-default-zone 2>/dev/null || echo public)"
  sudo firewall-cmd --permanent --zone="$ZONE" --add-port="${PORT}/tcp" || true
  sudo firewall-cmd --reload
  echo "firewalld: TCP ${PORT} open on zone ${ZONE}"
  firewall-cmd --list-ports
elif command -v ufw >/dev/null 2>&1; then
  sudo ufw allow "${PORT}/tcp"
  sudo ufw status | head -30
else
  sudo iptables -C INPUT -p tcp --dport "$PORT" -j ACCEPT 2>/dev/null || \
    sudo iptables -I INPUT -p tcp --dport "$PORT" -j ACCEPT
  echo "iptables: ACCEPT tcp dport ${PORT}"
  echo "Note: iptables rules may not persist across reboot without iptables-services/nft."
fi
echo "Test: curl -s http://$(hostname -I | awk '{print $1}'):${PORT}/health"
