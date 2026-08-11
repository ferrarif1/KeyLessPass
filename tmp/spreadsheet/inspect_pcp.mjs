import { FileBlob, SpreadsheetFile } from "@oai/artifact-tool";
import fs from "node:fs/promises";

const source = process.argv[2];
const workbook = await SpreadsheetFile.importXlsx(await FileBlob.load(source));
const overview = await workbook.inspect({
  kind: "workbook,sheet,table",
  maxChars: 12000,
  tableMaxRows: 8,
  tableMaxCols: 24,
  tableMaxCellChars: 120,
});
process.stdout.write(overview.ndjson + "\n");

if (process.argv[3]) {
  const sheet = workbook.worksheets.getItem("policy_websites");
  const values = sheet.getUsedRange(true).values;
  const [headers, ...rows] = values;
  const objects = rows.map((row) =>
    Object.fromEntries(headers.map((header, index) => [String(header), row[index] ?? null])),
  );
  await fs.writeFile(process.argv[3], JSON.stringify(objects, null, 2));
}
