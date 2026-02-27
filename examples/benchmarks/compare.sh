#!/bin/bash
# Compare FD vs Excalidraw benchmark metrics
# Usage: ./compare.sh

cd "$(dirname "$0")"

printf "%-24s %6s %8s %6s %8s %6s %6s\n" "Example" "FD ln" "FD bytes" "EX ln" "EX bytes" "Ratio" "Tokens"
printf "%-24s %6s %8s %6s %8s %6s %6s\n" "-------" "-----" "--------" "-----" "--------" "-----" "------"

for fd in *.fd; do
  base="${fd%.fd}"
  ex="${base}.excalidraw.json"
  [ -f "$ex" ] || continue

  fd_bytes=$(wc -c < "$fd" | tr -d ' ')
  fd_lines=$(wc -l < "$fd" | tr -d ' ')
  ex_bytes=$(wc -c < "$ex" | tr -d ' ')
  ex_lines=$(wc -l < "$ex" | tr -d ' ')

  ratio=$(echo "scale=1; $ex_bytes / $fd_bytes" | bc)
  fd_tok=$(cat "$fd" | tr -cs 'a-zA-Z0-9_@#' '\n' | wc -l | tr -d ' ')
  ex_tok=$(cat "$ex" | tr -cs 'a-zA-Z0-9_@#' '\n' | wc -l | tr -d ' ')
  tok_ratio=$(echo "scale=1; $ex_tok / $fd_tok" | bc)

  printf "%-24s %6s %8s %6s %8s %5sx %5sx\n" "$base" "$fd_lines" "$fd_bytes" "$ex_lines" "$ex_bytes" "$ratio" "$tok_ratio"
done

echo ""
echo "Ratio = Excalidraw bytes / FD bytes (higher = FD more concise)"
echo "Tokens = Excalidraw word tokens / FD word tokens"
