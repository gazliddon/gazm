focus soundcpu
wpset 400:soundcpu,4,w,,{ logerror "PIA W pc=%X cycle=%X addr=%X value=%X",pc,totalcycles,wpaddr,wpdata ; g }
wpset 400:soundcpu,4,r,,{ logerror "PIA R pc=%X cycle=%X addr=%X value=%X",pc,totalcycles,wpaddr,wpdata ; g }
g
