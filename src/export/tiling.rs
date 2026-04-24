// Splits an export job (e.g. 16K x 8K equirect) into 4K tiles, renders each
// with adjusted ray origin/UV offsets, stitches CPU-side. Handles supersample.
