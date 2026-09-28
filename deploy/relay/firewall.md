# Relay firewall

The relay needs **inbound TCP only**: TCP/5900 by default. Keep SSH access only
while you administer the VM, and restrict it to your own source range where
possible. Do not open QUIC/UDP on the relay: the relay is the fallback for
networks that block UDP; direct PCC connections use QUIC separately.

These examples accept new TCP relay connections at a conservative rate and
allow packets belonging to established connections. Adjust `5900` only if the
systemd environment file uses a different relay port.

## nftables

Add the following to the active `inet filter input` chain (or create an
equivalent chain in your nftables configuration):

```nft
ct state established,related accept
tcp dport 5900 ct state new limit rate 20/minute burst 10 packets accept
tcp dport 5900 ct state new drop
```

Persist it using your distribution's normal nftables configuration mechanism.

## iptables

```sh
sudo iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 5900 -m conntrack --ctstate NEW \
  -m limit --limit 20/minute --limit-burst 10 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 5900 -m conntrack --ctstate NEW -j DROP
```

Persist these rules using `iptables-persistent` or your operating system's
firewall service. OCI's security list must also permit TCP/5900; a host rule
cannot override a cloud firewall denial.
