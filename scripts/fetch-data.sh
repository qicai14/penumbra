#!/usr/bin/env bash
# Download the raw event data (kernels, PGDA rasters, LOLA DEMs). Resumable and idempotent.
# Saves to $PENUMBRA_DATA (default /d/DownloadHDD/penumbra-resource) and logs each finished
# file to MANIFEST.tsv there: url, bytes, sha256, date. Nothing is unpacked or processed.
# Usage: bash scripts/fetch-data.sh        (re-run any time; complete files are skipped)
set -u
dir="${PENUMBRA_DATA:-/d/DownloadHDD/penumbra-resource}"
mkdir -p "$dir" || exit 1
manifest="$dir/MANIFEST.tsv"
[ -f "$manifest" ] || printf 'url\tbytes\tsha256\tdate\n' > "$manifest"

# Small first (PLAN-DRAFT §5 checklist): kernels, then PSR, far field, AVGVISIB pair, then the DEM.
# Sources: NAIF generic_kernels, nyx-space ANISE data (facts-grill-1.md §2), PGDA products 90 and 69.
urls=(
  https://naif.jpl.nasa.gov/pub/naif/generic_kernels/lsk/naif0012.tls
  https://naif.jpl.nasa.gov/pub/naif/generic_kernels/fk/satellites/moon_de440_250416.tf
  http://public-data.nyxspace.com/anise/v0.7/moon_fk_de440.epa
  https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/moon_pa_de440_200625.bpc
  https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp
  https://pgda.gsfc.nasa.gov/data/MoonIllumination/AVGVISIB_85S_060M_201608.TIF
  https://pgda.gsfc.nasa.gov/data/MoonIllumination/AVGVISIB_85S_060M_201608_EARTH.TIF
  https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/LPSR_80S_20MPP_ADJ.TIF
  https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/LDEM_60S_240MPP_ADJ.TIF
  https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/LDEM_80S_20MPP_ADJ.TIF
)

fail=0
for url in "${urls[@]}"; do
  f="$dir/${url##*/}"
  want=$(curl -sIL --max-time 60 "$url" | tr -d '\r' | awk 'tolower($1)=="content-length:"{n=$2} END{print n}')
  if [ -z "$want" ]; then echo "FAIL  no size from server: $url"; fail=1; continue; fi
  have=$(stat -c %s "$f" 2>/dev/null || echo 0)
  if [ "$have" != "$want" ]; then
    echo "GET   ${url##*/} ($have / $want B)"
    curl -fsSL -C - --retry 10 --retry-delay 5 --retry-all-errors -o "$f" "$url"
    have=$(stat -c %s "$f" 2>/dev/null || echo 0)
    if [ "$have" != "$want" ]; then echo "FAIL  ${url##*/}: $have of $want B, re-run to resume"; fail=1; continue; fi
  fi
  if grep -q "^$url	$want	" "$manifest"; then echo "skip  ${url##*/} (complete)"; continue; fi
  sum=$(sha256sum "$f" | cut -d' ' -f1)
  grep -v "^$url	" "$manifest" > "$manifest.tmp"; mv "$manifest.tmp" "$manifest"
  printf '%s\t%s\t%s\t%s\n' "$url" "$want" "$sum" "$(date -u +%FT%TZ)" >> "$manifest"
  echo "ok    ${url##*/} $want B $sum"
done
exit $fail
