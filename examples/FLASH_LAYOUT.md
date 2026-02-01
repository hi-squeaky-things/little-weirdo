# Flash storage Key assignment

Capacity: Supports up to 9 x 99 slots.
Partition: 8 MB maximum storage spaces.

Data Format:

- All wavetable forms and samples are in 16-bit resolution.
- Each wavetable form consists of 600 cycles.
- Sampler samples has a sample rate of 44100 hertz, resulting in a storage size of 88200 (88.2 KB/s) bytes per second.

## Key layout

Key = NN_A_BCCu16

- NN = id of block = 0..99
- A = 1 = patch
- A = 2 = wavetable forms
- A = 3 = sampler samples
- B = 0..9 = bank 0 .. 9
- CC = 0..99 = slot 0 ... 99

## Examples

700 patches = 700 x 2048
200 waveforms = 200 x 2048
100 samples

| Key | Description | Key Example | Key Description |
|----------|----------|----------|----------|
| 0XX | Patches bank 0 | 000 | Patch bank 0 - patch 0 |
| 1XX | Patches bank 1 | 150 | Patch bank 1 - patch 50 |
| 6XX | Patches bank 6 | 699 | Patch bank 6 - patch 99 |
| 7XX | Wavetable Synthesizer samples| 700 | Wavetable Synthesizer samples bank 0 - waveform 0 |
| 8XX | Wavetable Synthesizer samples| 899 | Wavetable Synthesizer samples bank 1 - waveform 99 |
| 9XX | Sampler samples | 003_010 | Sampler samples bank 0 - sample 10 |