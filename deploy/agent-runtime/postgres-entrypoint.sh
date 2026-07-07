#!/usr/bin/env bash
# Ensure TCP clients on the container network can authenticate (dev self-host).
set -euo pipefail

PGDATA="${PGDATA:-/var/lib/postgresql/data}"
HBA_MARKER="# agent-runtime: allow container network (scram-sha-256, no SSL)"

if [[ -f "${PGDATA}/pg_hba.conf" ]] && ! grep -qF "${HBA_MARKER}" "${PGDATA}/pg_hba.conf"; then
  {
    echo ""
    echo "${HBA_MARKER}"
    echo "host all all all scram-sha-256"
  } >>"${PGDATA}/pg_hba.conf"
fi

exec /usr/local/bin/docker-entrypoint.sh "$@"
