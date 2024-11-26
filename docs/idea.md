- via uart een soort van shell waarmee je zaken kan testen
- aantal predefined wavetables 
- een taal om de patch te defineren

bijv.


osc_1 ~ sin (0)
osc_2 ~ sin (-10)
osc_3 ~ sin (10)

osc_4 ~ sqr (0)
 
osc_1 -> env_1
osc_2 -> env_1
osc_3 -> env_1

osc_4 -> env_2

env_1 ~ lin(100,50,100,10)
env_2 ~ lin(100,50,100,10)


mc_1 -> osc_1 
mc_1 -> osc_2 
mc_1 -> osc_3 

mc_2 -> osc_4

mc_1 -> env_1
mc_2 -> env_2