#Defender

defend.1 = 

	./asm6809/src/asm6809 -B src/phr6.src src/defa7.src src/defb6.src src/amode1.src\
	 		-s bin/defa7-defb6-amode1.sym -l bin/defa7-defb6-amode1.lst  -o bin/defa7-defb6-amode1.o

	# defend.1
	./ChainFilesToRom.py redlabel/defend.1 0x800\
		bin/defa7-defb6-amode1.o,0xb001,0x0000,0x0800,"defa7"
	echo "c9eb365411ca8452debe66e7b7657f44  redlabel/defend.1" | md5sum -c


MAPC
d000 Select bank (c000-cfff)
      0 = I/O
      1 = BANK 1 - roms 9 12
      2 = BANK 2 - roms 8 11
      3 = BANK 3 - roms 7 10
      7 = BANK 4 - roms 6 ??


This is a very dumb story about how I failed to achieve my goal of doing some hobby coding for the 6809 by putting in a shit load of effort

A while back I thought I'd like to try programming the 6809, an old 8 bit processors that never hit the heady heights of usage of the 6502 or Z80 but has always been held in high regard by many old-skool coders

And a CPU of particular interest to me as it's the beating heart of the holy trinity of early arcade games: Defender, Robotron and Stargate. Playing Defender at the Richmond Ice rink the early 80s put me on a path into the games industry. The lights!

I read up online a bit and saw that you could develop code for that early Williams Hardware using one of the many available 6809 cross assemblers and MAME as a dev target. Nice.

I had a look at MAME and didn't really like the dev environment. No source level debugging? I'm sure I could do better. So I started written a 6809 emulator :D

And that was fun. I used Rust, a system programming language with high level abstractions for a low cost. I know C++ a lot better but, you know, it's always nice to learn things.

I'd never written an emulator before and that was fun as well, though a bit painstaking. The 6809's amazing set of indexing modes is tricky to decode properly and creates a huge set of cases that need testing. I spent a bit of time on automating that, comparing my emu with other 

And I wrote a simple source level debugger.

So time to write some 6809! But I didn't like the 6809 cross-assemblers available. There's nothing wrong with them, there's something wrong with me :) So I decided to write an assembler. And that was fun, I learned a lot about how you write a parser, some tedious grinding on operator precedence.

To test that everything was working corrected I took the Defender source code, updated to assemble using AS6809, and converted it my assembler syntax to see if I could produce redlabel defender binaries that would work with MAME.

I found a -lot- of bugs in my code generation. It was cool being able to load the Defender ROMs as reference before assembling and error when my assembler went off course. That helped track down a lot of issues.

It took a while but now all works.

Having written a 6809 emulator and a 6809 assembler I'd say I know quite a lot about the 6809 despite haven written very little of it. Which is weird.

So now time to actually write some 6809? Erm... Soon! I'm going to add the 6800 as a target so I can assemble the sound ROM as well :D

The moral of the story is Follow Your Dream! Or main



