Defender

defend.1 = 

	./asm6809/src/asm6809 -B src/phr6.src src/defa7.src src/defb6.src src/amode1.src\
	 		-s bin/defa7-defb6-amode1.sym -l bin/defa7-defb6-amode1.lst  -o bin/defa7-defb6-amode1.o

	# defend.1
	./ChainFilesToRom.py redlabel/defend.1 0x800\
		bin/defa7-defb6-amode1.o,0xb001,0x0000,0x0800,"defa7"
	echo "c9eb365411ca8452debe66e7b7657f44  redlabel/defend.1" | md5sum -c
