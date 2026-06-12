#!/usr/bin/env bash
# Prints the standard memory summary block used in push reports.
set -euo pipefail

if ! command -v free >/dev/null 2>&1; then
  echo "Команда 'free' не найдена. Запустите скрипт в Linux/WSL окружении."
  exit 1
fi

read -r mem_total mem_used swap_total swap_used < <(
  free -m | awk '
    /^Mem:/ { mem_total = $2; mem_used = $3 }
    /^Swap:/ { swap_total = $2; swap_used = $3 }
    END { printf "%s %s %s %s\n", mem_total, mem_used, swap_total, swap_used }
  '
)

total_all=$((mem_total + swap_total))
used_all=$((mem_used + swap_used))

printf 'Память всего: %s MiB / %s MiB\n' "$total_all" "$used_all"
printf 'Физическая память всего: %s MiB / %s MiB\n' "$mem_total" "$mem_used"
printf 'Swap всего: %s MiB / %s MiB\n' "$swap_total" "$swap_used"
