focus soundcpu
bp f832:soundcpu,1,{ logerror "WMS_PROBE pc=%X a=%X b=%X x=%X ram0d=%X ram0e=%X ram0f=%X ram10=%X cycle=%X",pc,a,b,x,b@0d,b@0e,b@0f,b@10,totalcycles ; bpdisable 1 ; g }
bp f837:soundcpu,1,{ logerror "WMS_PROBE pc=%X a=%X b=%X x=%X ram0d=%X ram0e=%X ram0f=%X ram10=%X cycle=%X",pc,a,b,x,b@0d,b@0e,b@0f,b@10,totalcycles ; bpdisable 2 ; g }
bp fae0:soundcpu,1,{ logerror "WMS_PROBE pc=%X a=%X b=%X x=%X ram0d=%X ram0e=%X ram0f=%X ram10=%X cycle=%X",pc,a,b,x,b@0d,b@0e,b@0f,b@10,totalcycles ; bpdisable 3 ; g }
bp f83f:soundcpu,1,{ logerror "WMS_PROBE pc=%X a=%X b=%X x=%X s=%X cc=%X ram1b=%X ram13=%X ram14=%X ram1c=%X ram1d=%X cycle=%X",pc,a,b,x,s,cc,b@1b,b@13,b@14,b@1c,b@1d,totalcycles ; bpdisable 4 ; g }
g
