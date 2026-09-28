#!/usr/bin/env bash
# Provision or reuse a smallest ARM Always Free Ubuntu relay with OCI CLI.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: provision-oracle.sh [options]

Required values can be supplied by flags or environment variables:
  --tenancy-ocid OCID       OCI_TENANCY_OCID
  --user-ocid OCID          OCI_USER_OCID
  --compartment-ocid OCID   OCI_COMPARTMENT_OCID
  --ssh-public-key FILE     OCI_SSH_PUBLIC_KEY_FILE (a path)

Options:
  --region REGION           OCI_REGION (otherwise the OCI CLI profile region)
  --release-url URL         PCC_RELEASE_URL, Linux ARM64 .tar.gz release asset
  --token TOKEN             PCC_RELAY_TOKEN (generated when omitted)
  --instance-name NAME      PCC_INSTANCE_NAME (default: pcc-relay)
  --dry-run                 Print commands without executing them
  -h, --help                Show this help

The OCI CLI must already be configured with credentials for the supplied user.
USAGE
}

TENANCY_OCID=${OCI_TENANCY_OCID:-}
USER_OCID=${OCI_USER_OCID:-}
COMPARTMENT_OCID=${OCI_COMPARTMENT_OCID:-}
SSH_PUBLIC_KEY_FILE=${OCI_SSH_PUBLIC_KEY_FILE:-}
REGION=${OCI_REGION:-}
RELEASE_URL=${PCC_RELEASE_URL:-}
RELAY_TOKEN=${PCC_RELAY_TOKEN:-}
INSTANCE_NAME=${PCC_INSTANCE_NAME:-pcc-relay}
DRY_RUN=false

while (($#)); do
  case "$1" in
    --tenancy-ocid) TENANCY_OCID=$2; shift 2 ;;
    --user-ocid) USER_OCID=$2; shift 2 ;;
    --compartment-ocid) COMPARTMENT_OCID=$2; shift 2 ;;
    --ssh-public-key) SSH_PUBLIC_KEY_FILE=$2; shift 2 ;;
    --region) REGION=$2; shift 2 ;;
    --release-url) RELEASE_URL=$2; shift 2 ;;
    --token) RELAY_TOKEN=$2; shift 2 ;;
    --instance-name) INSTANCE_NAME=$2; shift 2 ;;
    --dry-run) DRY_RUN=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "error: unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

fail() { echo "error: $*" >&2; exit 1; }
for required in TENANCY_OCID USER_OCID COMPARTMENT_OCID SSH_PUBLIC_KEY_FILE; do
  [[ -n ${!required} ]] || fail "missing $required; pass its flag or set it in the environment"
done
[[ -f $SSH_PUBLIC_KEY_FILE ]] || fail "SSH public-key file does not exist: $SSH_PUBLIC_KEY_FILE"
[[ -n $RELEASE_URL ]] || fail "missing PCC_RELEASE_URL/--release-url: provide the released Linux ARM64 tar.gz asset URL"
[[ $RELEASE_URL =~ ^https://[^[:space:]]+$ ]] || fail "--release-url must be an HTTPS URL"
[[ $RELAY_TOKEN =~ ^[[:alnum:]_-]*$ ]] || fail "--token may contain only letters, digits, underscores, and hyphens"
command -v oci >/dev/null 2>&1 || fail "OCI CLI is not installed. Install it from https://docs.oracle.com/iaas/Content/API/SDKDocs/cliinstall.htm, run 'oci setup config', then retry."
command -v ssh >/dev/null 2>&1 || fail "ssh is required to install the relay after provisioning"
command -v scp >/dev/null 2>&1 || fail "scp is required to install the relay after provisioning"

OCI=(oci)
[[ -n $REGION ]] && OCI+=(--region "$REGION")
run() {
  printf '+ '
  printf '%q ' "$@"
  printf '\n'
  "$DRY_RUN" || "$@"
}
raw() {
  if "$DRY_RUN"; then
    run "${OCI[@]}" "$@"
    return 0
  fi
  "${OCI[@]}" "$@"
}

if [[ -z $RELAY_TOKEN ]]; then
  command -v openssl >/dev/null 2>&1 || fail "openssl is required to generate a relay token; pass --token instead"
  RELAY_TOKEN=$(openssl rand -hex 32)
fi

# The default resource names make reruns idempotent and avoid duplicate free VMs.
VCN_NAME="${INSTANCE_NAME}-vcn"
IG_NAME="${INSTANCE_NAME}-ig"
ROUTE_TABLE_NAME="${INSTANCE_NAME}-rt"
SECURITY_LIST_NAME="${INSTANCE_NAME}-sl"
SUBNET_NAME="${INSTANCE_NAME}-subnet"

run "${OCI[@]}" iam tenancy get --tenancy-id "$TENANCY_OCID" >/dev/null
run "${OCI[@]}" iam user get --user-id "$USER_OCID" >/dev/null

if "$DRY_RUN"; then
  # These are the OCI CLI invocations the no-existing-resources path uses.
  # Dependent OCIDs are output from the immediately preceding create/list call.
  VCN_ID='<vcn-ocid>'
  IG_ID='<internet-gateway-ocid>'
  ROUTE_TABLE_ID='<route-table-ocid>'
  SECURITY_LIST_ID='<security-list-ocid>'
  SUBNET_ID='<subnet-ocid>'
  AD='<availability-domain-name>'
  IMAGE_ID='<ubuntu-image-ocid>'
  run "${OCI[@]}" network vcn list --compartment-id "$COMPARTMENT_OCID" --display-name "$VCN_NAME" --query 'data[0].id' --raw-output
  run "${OCI[@]}" network vcn create --compartment-id "$COMPARTMENT_OCID" --display-name "$VCN_NAME" --cidr-block 10.42.0.0/16 --dns-label pccrelay --wait-for-state AVAILABLE --query 'data.id' --raw-output
  run "${OCI[@]}" network internet-gateway list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$IG_NAME" --query 'data[0].id' --raw-output
  run "${OCI[@]}" network internet-gateway create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$IG_NAME" --is-enabled true --query 'data.id' --raw-output
  run "${OCI[@]}" network route-table list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$ROUTE_TABLE_NAME" --query 'data[0].id' --raw-output
  run "${OCI[@]}" network route-table create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$ROUTE_TABLE_NAME" --route-rules "[{\"cidrBlock\":\"0.0.0.0/0\",\"networkEntityId\":\"$IG_ID\"}]" --query 'data.id' --raw-output
  run "${OCI[@]}" network security-list list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SECURITY_LIST_NAME" --query 'data[0].id' --raw-output
  run "${OCI[@]}" network security-list create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SECURITY_LIST_NAME" --egress-security-rules '[{"destination":"0.0.0.0/0","protocol":"all"}]' --ingress-security-rules '[{"protocol":"6","source":"0.0.0.0/0","tcpOptions":{"destinationPortRange":{"min":22,"max":22}}},{"protocol":"6","source":"0.0.0.0/0","tcpOptions":{"destinationPortRange":{"min":5900,"max":5900}}}]' --query 'data.id' --raw-output
  run "${OCI[@]}" network subnet list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SUBNET_NAME" --query 'data[0].id' --raw-output
  run "${OCI[@]}" network subnet create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SUBNET_NAME" --cidr-block 10.42.0.0/24 --route-table-id "$ROUTE_TABLE_ID" --security-list-ids "[\"$SECURITY_LIST_ID\"]" --prohibit-public-ip-on-vnic false --query 'data.id' --raw-output
  run "${OCI[@]}" iam availability-domain list --compartment-id "$TENANCY_OCID" --query 'data[0].name' --raw-output
  run "${OCI[@]}" compute image list --compartment-id "$TENANCY_OCID" --operating-system 'Canonical Ubuntu' --operating-system-version '22.04' --shape VM.Standard.A1.Flex --sort-by TIMECREATED --sort-order DESC --query 'data[0].id' --raw-output
  run "${OCI[@]}" compute instance list --compartment-id "$COMPARTMENT_OCID" --display-name "$INSTANCE_NAME" --query 'data[?"lifecycle-state"!=`TERMINATED`][0].id' --raw-output
  run "${OCI[@]}" compute instance list --compartment-id "$COMPARTMENT_OCID" --all --query 'sum(data[?shape==`VM.Standard.A1.Flex` && "lifecycle-state"!=`TERMINATED`]."shape-config".ocpus)' --raw-output
  run "${OCI[@]}" compute instance launch --compartment-id "$COMPARTMENT_OCID" --availability-domain "$AD" --display-name "$INSTANCE_NAME" --shape VM.Standard.A1.Flex --shape-config '{"ocpus":1,"memoryInGBs":1}' --image-id "$IMAGE_ID" --subnet-id "$SUBNET_ID" --assign-public-ip true --ssh-authorized-keys-file "$SSH_PUBLIC_KEY_FILE" --wait-for-state RUNNING --max-wait-seconds 900
  run "${OCI[@]}" compute vnic-attachment list --compartment-id "$COMPARTMENT_OCID" --instance-id '<instance-ocid>' --query 'data[0]."vnic-id"' --raw-output
  run "${OCI[@]}" network vnic get --vnic-id '<vnic-ocid>' --query 'data."public-ip"' --raw-output
  echo "Dry run complete: no OCI, SSH, or SCP command was executed."
  exit 0
fi

VCN_ID=$(raw network vcn list --compartment-id "$COMPARTMENT_OCID" --display-name "$VCN_NAME" --query 'data[0].id' --raw-output)
if [[ $VCN_ID == "None" || -z $VCN_ID ]]; then
  VCN_ID=$(raw network vcn create --compartment-id "$COMPARTMENT_OCID" --display-name "$VCN_NAME" --cidr-block 10.42.0.0/16 --dns-label pccrelay --wait-for-state AVAILABLE --query 'data.id' --raw-output)
fi
IG_ID=$(raw network internet-gateway list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$IG_NAME" --query 'data[0].id' --raw-output)
if [[ $IG_ID == "None" || -z $IG_ID ]]; then
  IG_ID=$(raw network internet-gateway create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$IG_NAME" --is-enabled true --query 'data.id' --raw-output)
fi
ROUTE_TABLE_ID=$(raw network route-table list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$ROUTE_TABLE_NAME" --query 'data[0].id' --raw-output)
if [[ $ROUTE_TABLE_ID == "None" || -z $ROUTE_TABLE_ID ]]; then
  ROUTE_TABLE_ID=$(raw network route-table create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$ROUTE_TABLE_NAME" --route-rules "[{\"cidrBlock\":\"0.0.0.0/0\",\"networkEntityId\":\"$IG_ID\"}]" --query 'data.id' --raw-output)
fi
SECURITY_LIST_ID=$(raw network security-list list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SECURITY_LIST_NAME" --query 'data[0].id' --raw-output)
if [[ $SECURITY_LIST_ID == "None" || -z $SECURITY_LIST_ID ]]; then
  SECURITY_LIST_ID=$(raw network security-list create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SECURITY_LIST_NAME" --egress-security-rules '[{"destination":"0.0.0.0/0","protocol":"all"}]' --ingress-security-rules '[{"protocol":"6","source":"0.0.0.0/0","tcpOptions":{"destinationPortRange":{"min":22,"max":22}}},{"protocol":"6","source":"0.0.0.0/0","tcpOptions":{"destinationPortRange":{"min":5900,"max":5900}}}]' --query 'data.id' --raw-output)
fi
SUBNET_ID=$(raw network subnet list --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SUBNET_NAME" --query 'data[0].id' --raw-output)
if [[ $SUBNET_ID == "None" || -z $SUBNET_ID ]]; then
  SUBNET_ID=$(raw network subnet create --compartment-id "$COMPARTMENT_OCID" --vcn-id "$VCN_ID" --display-name "$SUBNET_NAME" --cidr-block 10.42.0.0/24 --route-table-id "$ROUTE_TABLE_ID" --security-list-ids "[\"$SECURITY_LIST_ID\"]" --prohibit-public-ip-on-vnic false --query 'data.id' --raw-output)
fi
AD=$(raw iam availability-domain list --compartment-id "$TENANCY_OCID" --query 'data[0].name' --raw-output)
IMAGE_ID=$(raw compute image list --compartment-id "$TENANCY_OCID" --operating-system 'Canonical Ubuntu' --operating-system-version '22.04' --shape VM.Standard.A1.Flex --sort-by TIMECREATED --sort-order DESC --query 'data[0].id' --raw-output)
INSTANCE_ID=$(raw compute instance list --compartment-id "$COMPARTMENT_OCID" --display-name "$INSTANCE_NAME" --query 'data[?"lifecycle-state"!=`TERMINATED`][0].id' --raw-output)

# Oracle Always Free permits four A1 OCPUs and 24 GB RAM per tenancy. This VM uses one of each.
USED_A1_OCPUS=$(raw compute instance list --compartment-id "$COMPARTMENT_OCID" --all --query 'sum(data[?shape==`VM.Standard.A1.Flex` && "lifecycle-state"!=`TERMINATED`]."shape-config".ocpus)' --raw-output)
USED_A1_OCPUS=${USED_A1_OCPUS:-0}
if [[ $INSTANCE_ID == "None" || -z $INSTANCE_ID ]] && (( ${USED_A1_OCPUS%.*} >= 4 )); then
  fail "the Always Free A1 limit is exhausted (${USED_A1_OCPUS}/4 OCPUs). Terminate or resize an existing A1 instance, or use a paid VM."
fi

if [[ $INSTANCE_ID == "None" || -z $INSTANCE_ID || $INSTANCE_ID == '<existing-or-created-instance-ocid>' ]]; then
  LAUNCH=(compute instance launch --compartment-id "$COMPARTMENT_OCID" --availability-domain "$AD" --display-name "$INSTANCE_NAME" --shape VM.Standard.A1.Flex --shape-config '{"ocpus":1,"memoryInGBs":1}' --image-id "$IMAGE_ID" --subnet-id "$SUBNET_ID" --assign-public-ip true --ssh-authorized-keys-file "$SSH_PUBLIC_KEY_FILE" --wait-for-state RUNNING --max-wait-seconds 900)
  if ! LAUNCH_OUTPUT=$(raw "${LAUNCH[@]}" 2>&1); then
    case $LAUNCH_OUTPUT in
      *"Out of host capacity"*|*"OutOfCapacity"*) fail "Always Free A1 capacity is unavailable in this region/availability domain. Retry later, choose another region, or use the roughly $4/month paid alternative." ;;
      *"LimitExceeded"*|*"limit"*) fail "the Always Free shape limit is exhausted. Remove an existing A1 instance or use a paid VM. OCI said: $LAUNCH_OUTPUT" ;;
      *) fail "OCI failed to launch the instance: $LAUNCH_OUTPUT" ;;
    esac
  else
    # Re-query avoids parsing launch JSON with an additional dependency.
    INSTANCE_ID=$(raw compute instance list --compartment-id "$COMPARTMENT_OCID" --display-name "$INSTANCE_NAME" --query 'data[?"lifecycle-state"==`RUNNING`][0].id' --raw-output)
  fi
else
  echo "Reusing existing relay instance: $INSTANCE_ID"
fi


VNIC_ID=$(raw compute vnic-attachment list --compartment-id "$COMPARTMENT_OCID" --instance-id "$INSTANCE_ID" --query 'data[0]."vnic-id"' --raw-output)
PUBLIC_IP=$(raw network vnic get --vnic-id "$VNIC_ID" --query 'data."public-ip"' --raw-output)
[[ $PUBLIC_IP != "None" && -n $PUBLIC_IP ]] || fail "instance has no public IP; verify the subnet permits public IP assignment"

REMOTE="ubuntu@$PUBLIC_IP"
# Download on the host so GitHub/CDN redirects are handled there; archive must contain the release binary.
ssh -o StrictHostKeyChecking=accept-new "$REMOTE" "sudo apt-get update && sudo apt-get install -y ca-certificates curl && curl -fL '$RELEASE_URL' -o /tmp/pcc.tar.gz; sudo useradd --system --home /var/lib/pcc-relay --create-home --shell /usr/sbin/nologin pcc-relay 2>/dev/null || true"
ssh "$REMOTE" "set -e; sudo tar -xzf /tmp/pcc.tar.gz -C /tmp; binary=\\$(find /tmp -type f \\( -name pixel-change-check-client -o -name pcc \\) -print -quit); test -n \\\"\\$binary\\\"; sudo install -m 0755 \\\"\\$binary\\\" /usr/local/bin/pcc; sudo mkdir -p /etc/pcc-relay /var/lib/pcc-relay; sudo chown pcc-relay:pcc-relay /var/lib/pcc-relay"
scp deploy/relay/pcc-relay.service "$REMOTE:/tmp/pcc-relay.service"
ssh "$REMOTE" "sudo install -m 0644 /tmp/pcc-relay.service /etc/systemd/system/pcc-relay.service && sudo sh -c 'printf %s\\n PCC_RELAY_LISTEN=0.0.0.0:5900 PCC_RELAY_TOKEN=$RELAY_TOKEN > /etc/pcc-relay/pcc-relay.env' && sudo chmod 0600 /etc/pcc-relay/pcc-relay.env && sudo systemctl daemon-reload && sudo systemctl enable --now pcc-relay"
RELAY_INFO=$(ssh "$REMOTE" "sudo journalctl -u pcc-relay --no-pager -n 100")
FINGERPRINT=$(printf '%s\n' "$RELAY_INFO" | sed -n 's/.*[Ff]ingerprint[^:]*: *//p' | tail -n 1)
[[ -n $FINGERPRINT ]] || fail "relay started but its certificate fingerprint was not found in the journal; run 'sudo journalctl -u pcc-relay -n 100' on $PUBLIC_IP"

cat <<EOF
Relay is running at: $PUBLIC_IP:5900

Copy this to every sharer:
  pcc share --relay $PUBLIC_IP:5900 --relay-pin $FINGERPRINT --token $RELAY_TOKEN

Copy this to every viewer:
  pcc view --relay $PUBLIC_IP:5900 --pin $FINGERPRINT --token $RELAY_TOKEN
EOF
