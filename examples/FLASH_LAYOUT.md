# Flash storage Key assignment

Data Format:

- All wavetable forms and samples are in 16-bit resolution.
- Each wavetable form consists of 600 cycles.
- Sampler samples has a sample rate of 44100 hertz, resulting in a storage size of 88200 (88.2 KB/s) bytes per second.

## Key layout

Key = NN_ACCu16

- NN = id of block = 0..64 (max!)
- A = 0..6 = patch
- A = 7..8 = wavetable forms
- A = 9 = sampler samples
- CC = 0..99 = slot 0 ... 99

## Examples

| Key | Description | Key Example | Key Description |
|----------|----------|----------|----------|
| 0XX | Patches bank 0 | 000 | Patch bank 0 - patch 0 |
| 1XX | Patches bank 1 | 150 | Patch bank 1 - patch 50 |
| 6XX | Patches bank 6 | 699 | Patch bank 6 - patch 99 |
| 7XX | Wavetable Synthesizer samples| 700 | Wavetable Synthesizer samples bank 0 - waveform 0 |
| 8XX | Wavetable Synthesizer samples| 899 | Wavetable Synthesizer samples bank 1 - waveform 99 |
| 9XX | Sampler samples | 910 | Sampler samples bank 0 - sample 10 |