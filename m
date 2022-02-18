cargo run --bin gasm -- -v --star-comments --ignore-relative-offset-errors --max-errors 50 \
../defender/gasmsrc/macros.68 \
../defender/gasmsrc/phr6.src \
../defender/gasmsrc/defa7.src \
../defender/gasmsrc/defb6.src \
../defender/gasmsrc/amode1.src \
--as6809-lst ../defender/bin/defa7-defb6.lst \
--as6809-sym ../defender/bin/defa7-defb6.sym
