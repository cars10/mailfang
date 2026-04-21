#!/bin/bash

SERVER="127.0.0.1"
PORT="2525"
FROM="sender@example.com"

MAX_JOBS=4
job_count=0

run_swaks() {
    swaks --server "$SERVER" \
          --port "$PORT" \
          --from "$FROM" \
          --to "$1" \
          --header "Subject: $2" \
          --body "$3" \
          >/dev/null 2>&1 &
    
    ((job_count++))

    if (( job_count >= MAX_JOBS )); then
        wait -n   # wait for one job to finish
        ((job_count--))
    fi
}

# ===== GENERATE USERS =====

USERS_10=()
for i in $(seq 1 10); do
    USERS_10+=("user${i}@example.com")
done

USERS_1000=()
for i in $(seq 1 1000); do
    USERS_1000+=("bulkuser${i}@example.com")
done

# ===== PART 1 =====
echo "Sending 1000 emails to EACH of 10 users..."

for TO in "${USERS_10[@]}"; do
    for i in $(seq 1 1000); do
        SUBJECT="Load Test A - ${TO} - Email #$i"
        BODY="This is email #$i sent to $TO"
        run_swaks "$TO" "$SUBJECT" "$BODY"
    done
done

wait
job_count=0

echo "Finished Part 1."

# ===== PART 2 =====
echo "Sending 5 emails to EACH of 1000 users..."

for TO in "${USERS_1000[@]}"; do
    for i in $(seq 1 5); do
        SUBJECT="Load Test B - ${TO} - Email #$i"
        BODY="This is email #$i sent to $TO"
        run_swaks "$TO" "$SUBJECT" "$BODY"
    done
done

wait

echo "All done."
