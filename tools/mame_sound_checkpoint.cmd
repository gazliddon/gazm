focus soundcpu
trace /tmp/mame-sound-checkpoint-trace.log,soundcpu,,tracesym pc
bp fc8c:soundcpu,1,{ logerror "WMS_SNAPSHOT pc=%X a=%X b=%X x=%X s=%X cc=%X totalcycles=%X",pc,a,b,x,s,cc,totalcycles ; save /tmp/mame-sound-ram.bin,0:soundcpu,100 ; save /tmp/mame-sound-pia.bin,400:soundcpu,4 ; bpdisable 1 ; g }
g
