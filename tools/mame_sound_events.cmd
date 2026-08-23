trace /tmp/mame-sound-events-trace.log,maincpu,,tracesym pc
wpset c80e:maincpu,1,w,(b@c80f & 4),{ tracelog "WMS_EVENT main_cycles=%X value=%02X\\n",totalcycles,wpdata ; g }
g
