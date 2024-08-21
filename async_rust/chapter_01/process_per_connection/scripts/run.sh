#!/usr/bin/env bash

# navigate to directory
SCRIPTPATH="$( cd "$(dirname "$0")" ; pwd -P )"
cd "$SCRIPTPATH"

cd ..

# Function to get current time in milliseconds
current_time_millis() {
  echo $(($(date +%s) * 1000 + $(date +%3N)))
}

# Start a timer in milliseconds
# start_time=$(current_time_millis)
# echo "Start time: $start_time milliseconds"

# Run the background tasks
./server_bin &
pid1=$!

./server_bin &
pid2=$!

./server_bin &
pid3=$!

./server_bin &
pid4=$!

# Wait for all background tasks to complete and capture their exit codes
wait $pid1
exit_code1=$?

wait $pid2
exit_code2=$?

wait $pid3
exit_code3=$?

wait $pid4
exit_code4=$?

# Calculate the elapsed time in milliseconds
# end_time=$(current_time_millis)
# elapsed_time=$((end_time - start_time))

# echo "End time: $end_time milliseconds"

# Print the outcomes and timing information
echo "Task 1 (PID $pid1) exited with code $exit_code1"
echo "Task 2 (PID $pid2) exited with code $exit_code2"
echo "Task 3 (PID $pid3) exited with code $exit_code3"
echo "Task 4 (PID $pid4) exited with code $exit_code4"

# echo "Total time elapsed: $elapsed_time milliseconds"
