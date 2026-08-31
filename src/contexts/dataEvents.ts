export const DATA_RELATIONS_CHANGED = "azriel:data-relations-changed";
export const notifyDataRelationsChanged = () => window.dispatchEvent(new Event(DATA_RELATIONS_CHANGED));
