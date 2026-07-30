import { readFile } from "node:fs/promises";
import path from "node:path";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";
import postcss from "postcss";

const traverse = traverseModule.default ?? traverseModule;
const JS_EXTENSIONS = new Set([".js", ".jsx", ".mjs", ".cjs"]);
const RAW_COLOR =
  /(?:^|[\s(:,])#[0-9a-f]{3,8}\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\s*\(/i;

function staticKey(node) {
  if (!node || node.computed) return null;
  if (node.key?.type === "Identifier") return node.key.name;
  if (node.key?.type === "StringLiteral") return node.key.value;
  return null;
}

export async function scanRawColors(filename, reject) {
  const extension = path.extname(filename);
  const source = await readFile(filename, "utf8");
  if (extension === ".css") {
    let tree;
    try {
      tree = postcss.parse(source, { from: filename });
    } catch (error) {
      reject("RG-PARSE", `${filename}: ${error.message}`);
    }
    tree.walkDecls((declaration) => {
      if (
        RAW_COLOR.test(declaration.value) ||
        /(?:color|background|border|fill|stroke)/i.test(declaration.prop)
      ) {
        reject(
          "RG-RAW-COLOR",
          `${filename}:${declaration.source.start.line} contains a raw color`,
        );
      }
    });
    return;
  }
  if (!JS_EXTENSIONS.has(extension)) return;

  let ast;
  try {
    ast = parse(source, {
      sourceType: "module",
      plugins: ["jsx", "importAttributes", "topLevelAwait"],
    });
  } catch (error) {
    reject("RG-PARSE", `${filename}: ${error.message}`);
  }
  traverse(ast, {
    StringLiteral(stringPath) {
      if (RAW_COLOR.test(stringPath.node.value)) {
        reject(
          "RG-RAW-COLOR",
          `${filename}:${stringPath.node.loc.start.line} contains a raw color`,
        );
      }
    },
    TemplateElement(templatePath) {
      if (RAW_COLOR.test(templatePath.node.value.raw)) {
        reject(
          "RG-RAW-COLOR",
          `${filename}:${templatePath.node.loc.start.line} contains a raw color`,
        );
      }
    },
    JSXAttribute(attributePath) {
      if (
        attributePath.node.name.type !== "JSXIdentifier" ||
        attributePath.node.name.name !== "style" ||
        attributePath.node.value?.type !== "JSXExpressionContainer" ||
        attributePath.node.value.expression.type !== "ObjectExpression"
      ) {
        return;
      }
      for (const property of attributePath.node.value.expression.properties) {
        const key = staticKey(property);
        if (!key || /(?:color|background|border|fill|stroke)/i.test(key)) {
          reject(
            "RG-RAW-COLOR",
            `${filename}:${attributePath.node.loc.start.line} contains an inline color style`,
          );
        }
      }
    },
  });
}
