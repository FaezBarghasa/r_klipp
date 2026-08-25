#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# test_slint_docker.sh
# 
# Builds and starts the Slint UI test container via Docker Compose.
# Waits for the container to reach a healthy state or complete execution.
# Inspects container logs for Slint rendering panics, segmentation faults,
# or UI test failures, and exits accordingly.
# ==============================================================================

SERVICE_NAME="slint-ui-test"
MAX_WAIT_SEC="${DOCKER_WAIT_TIMEOUT:-90}"

# Resolve docker compose command (supports both v2 'docker compose' and v1/v2 'docker-compose')
if docker compose version >/dev/null 2>&1; then
    COMPOSE_CMD="docker compose"
elif command -v docker-compose >/dev/null 2>&1; then
    COMPOSE_CMD="docker-compose"
else
    echo "[-] Error: Neither 'docker compose' nor 'docker-compose' found in PATH." >&2
    exit 1
fi

cleanup() {
    echo "[*] Cleaning up Docker Compose resources..."
    ${COMPOSE_CMD} down --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

echo "[*] Step 1: Building and launching Docker container (${COMPOSE_CMD} up --build)..."
${COMPOSE_CMD} up --build -d

echo "[*] Step 2: Waiting for container to be healthy or complete execution (Timeout: ${MAX_WAIT_SEC}s)..."
CONTAINER_ID=$(${COMPOSE_CMD} ps -q "${SERVICE_NAME}" | head -n 1)

if [ -z "${CONTAINER_ID}" ]; then
    echo "[-] Error: Could not determine container ID for service '${SERVICE_NAME}'." >&2
    exit 1
fi

ELAPSED=0
STATUS="starting"
while [ "${ELAPSED}" -lt "${MAX_WAIT_SEC}" ]; do
    INSPECT_STATUS=$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}' "${CONTAINER_ID}" 2>/dev/null || echo "unknown")
    RUNNING_STATE=$(docker inspect --format '{{.State.Status}}' "${CONTAINER_ID}" 2>/dev/null || echo "unknown")
    
    if [ "${INSPECT_STATUS}" = "healthy" ]; then
        STATUS="healthy"
        echo "[+] Container reached healthy state."
        break
    elif [ "${RUNNING_STATE}" = "exited" ]; then
        STATUS="exited"
        echo "[*] Container has finished running."
        break
    fi
    
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

echo ""
echo "==================== DOCKER CONTAINER LOGS ===================="
LOGS=$(${COMPOSE_CMD} logs "${SERVICE_NAME}" 2>&1)
echo "${LOGS}"
echo "================================================================"
echo ""

# Scan logs for panic signatures, rendering crashes, or UI test failures
FAILURES=0

if echo "${LOGS}" | grep -Ei "(panicked at|fatal runtime error|Slint rendering error|UI test failure|SIGSEGV|core dumped|failed to initialize rendering backend)" >/dev/null; then
    echo "[-] FAILURE: Detected panic, rendering crash, or UI test failure in container logs!"
    FAILURES=1
fi

# Check container exit code if exited
if [ "${STATUS}" = "exited" ]; then
    EXIT_CODE=$(docker inspect --format '{{.State.ExitCode}}' "${CONTAINER_ID}" 2>/dev/null || echo "1")
    if [ "${EXIT_CODE}" != "0" ]; then
        echo "[-] FAILURE: Container exited with non-zero exit code: ${EXIT_CODE}"
        FAILURES=1
    fi
elif [ "${STATUS}" != "healthy" ]; then
    echo "[-] FAILURE: Container did not become healthy within ${MAX_WAIT_SEC} seconds (Current status: ${STATUS})."
    FAILURES=1
fi

if [ "${FAILURES}" -eq 0 ]; then
    echo "[+] SUCCESS: Slint UI Docker test completed successfully without panics or rendering failures."
    exit 0
else
    exit 1
fi
