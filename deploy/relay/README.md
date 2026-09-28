# Oracle Always Free relay

Oracle Cloud's Always Free allowance includes ARM compute: up to four A1 OCPUs
and 24 GB of RAM total. This guide deliberately asks for only **1 OCPU and 1
GB**, which is enough for a small relay. It is free only while the account and
chosen resources remain within Oracle's current Always Free terms.

The catch is real: Oracle signup often requires a payment-card verification,
account activation can take time, and A1 host capacity is frequently unavailable
in popular regions. If a relay must be dependable, a small paid VM (roughly
**$4/month**, depending on region/provider) is the practical alternative.

## From zero to relay

1. Create an OCI account, complete its verification, install and configure the
   [OCI CLI](https://docs.oracle.com/iaas/Content/API/SDKDocs/cliinstall.htm),
   and choose a region with `VM.Standard.A1.Flex` capacity.
2. In OCI, copy the tenancy OCID, user OCID, and the compartment OCID where the
   VM should live. Create an SSH key pair if needed:

   ```sh
   ssh-keygen -t ed25519 -f ~/.ssh/pcc-relay
   ```

3. Obtain a released **Linux ARM64** archive URL. The Oracle A1 VM cannot run
   the x86_64 Linux release; publish or select the corresponding ARM64 asset.
4. Run the provisioner from the repository root. Start with `--dry-run`; it
   prints every OCI command without changing your account.

   ```sh
   export OCI_TENANCY_OCID=ocid1.tenancy.oc1..example
   export OCI_USER_OCID=ocid1.user.oc1..example
   export OCI_COMPARTMENT_OCID=ocid1.compartment.oc1..example
   export OCI_SSH_PUBLIC_KEY_FILE="$HOME/.ssh/pcc-relay.pub"
   export PCC_RELEASE_URL=https://github.com/OWNER/REPO/releases/download/vX.Y.Z/pcc-linux-aarch64.tar.gz

   deploy/relay/provision-oracle.sh --dry-run
   deploy/relay/provision-oracle.sh
   ```

   The script creates or reuses a named VCN, public subnet, and relay instance.
   It opens SSH and relay TCP/5900 at the OCI security list, then installs the
   release, configures `pcc-relay.service`, and prints the exact `pcc share`
   and `pcc view` commands. Re-running it reuses `pcc-relay` rather than
   creating another VM. Pass `--instance-name` to use an isolated name.
5. Apply host firewall rules from [firewall.md](firewall.md), then confirm:

   ```sh
   ssh -i ~/.ssh/pcc-relay ubuntu@PUBLIC_IP 'sudo systemctl status pcc-relay'
   ```

## Security model

The relay accepts TCP/TLS connections and uses a relay token to authorize a
session. Clients pin the relay certificate fingerprint (`--relay-pin`), so they
can reject a substituted relay certificate. Treat both the token and pin as
connection secrets: send them only to intended participants and rotate the
token by editing `/etc/pcc-relay/pcc-relay.env` and restarting the service.

A relay operator can observe the server IP, connection timing, peer IPs, byte
counts, and the token presented to the relay. The current relay forwards framed
application bytes; it does not decode screen frames, but that is **not** a
promise of end-to-end encryption. A privileged operator who controls the relay
or endpoint environment can log traffic or replace software. Do not use an
untrusted relay for sensitive material until end-to-end encryption is available.
