import { build } from "htmlpress"

const report = await build("docs")
console.log(`built ${report.pages} pages`)
