const pluginName = 'TypescriptSchema2JSON';
const fs = require('fs');
const { glob } = require('glob');

class TypescriptSchema2JSON {

	constructor({ source, dest }) {
		this.source = source
		this.dest = dest
	}

	compileTS() {

		const wasm = require(`${__dirname}/pkg/`);

		const results = [];
		glob(this.source).then(matchingFiles => {
			for (const filePath of matchingFiles) {
				let fileContents = fs.readFileSync(filePath, { encoding: 'utf8' });
				let resultingObject = JSON.parse(wasm.parse(fileContents));
				if (resultingObject.class_name != "") {
					results.push(JSON.parse(wasm.parse(fileContents)))
				}
			}
			const output = JSON.stringify(results, null, 2);
			fs.writeFileSync(this.dest, output);
		});
	}

	apply(compiler) {
		compiler.hooks.run.tap(pluginName, (_) => {
			console.log(`${pluginName}: Compiling Typescript...`);
			this.compileTS()
		});
	}
}

module.exports = TypescriptSchema2JSON;
