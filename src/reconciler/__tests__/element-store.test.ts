import { ElementStore } from "../element-store";

const elementStore = new ElementStore();

console.log("Running element store tests...");

elementStore.reset();

console.log("Test 1: createElement should generate unique IDs");
const id1 = elementStore.createElement("div");
const id2 = elementStore.createElement("div");
console.log(`  id1 = ${id1}, id2 = ${id2}`);
console.log(`  IDs are unique: ${id1 !== id2}`);

console.log("\nTest 2: createElement should store element data");
const id3 = elementStore.createElement("div", "test text");
const element = elementStore.getElement(id3);
console.log(`  element.type = ${element?.type}`);
console.log(`  element.text = ${element?.text}`);
console.log(`  Pass: ${element?.type === "div" && element?.text === "test text"}`);

console.log("\nTest 3: appendChild should create parent-child relationship");
const parentId = elementStore.createElement("div");
const childId = elementStore.createElement("text", "child text");

elementStore.appendChild(parentId, childId);

const parent = elementStore.getElement(parentId);
console.log(`  Parent children count: ${parent?.children.length}`);
console.log(`  Child in parent: ${parent?.children.includes(childId)}`);
console.log(`  Pass: ${parent?.children.includes(childId)}`);

console.log("\nTest 4: removeChild should remove child from parent");
const parentId2 = elementStore.createElement("div");
const childId2 = elementStore.createElement("text", "child text");

elementStore.appendChild(parentId2, childId2);
elementStore.removeChild(parentId2, childId2);

const parent2 = elementStore.getElement(parentId2);
console.log(`  Parent children count: ${parent2?.children.length}`);
console.log(`  Child removed: ${!parent2?.children.includes(childId2)}`);
console.log(`  Pass: ${!parent2?.children.includes(childId2)}`);

console.log("\nTest 5: getRoot should throw error when no root set");
try {
  elementStore.getRoot();
  console.log("  FAILED: Should have thrown error");
} catch (e: any) {
  console.log(`  Pass: Threw error as expected: ${e.message}`);
}

console.log("\nAll element store tests passed!");
