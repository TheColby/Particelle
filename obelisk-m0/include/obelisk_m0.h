#ifndef OBELISK_M0_H
#define OBELISK_M0_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ObeliskM0Handle ObeliskM0Handle;

ObeliskM0Handle *obelisk_m0_create(double sample_rate, size_t max_voices);
void obelisk_m0_destroy(ObeliskM0Handle *handle);
bool obelisk_m0_send_midi3(ObeliskM0Handle *handle, unsigned char status,
                           unsigned char data1, unsigned char data2);
bool obelisk_m0_process_stereo(ObeliskM0Handle *handle, float *output,
                               size_t frames);
/* Applies 128 positive frequency values at the next block boundary. */
bool obelisk_m0_set_tuning_table(ObeliskM0Handle *handle,
                                 const double *frequencies_hz);
void obelisk_m0_clear_tuning_table(ObeliskM0Handle *handle);
void obelisk_m0_all_notes_off(ObeliskM0Handle *handle);

#ifdef __cplusplus
}
#endif

#endif
