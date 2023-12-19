all: wasm pyo3

wasm:
	wasm-pack build --dev --target web

pyo3:
	maturin build -F pyo3
	
distclean:
	rm -rf ./pkg