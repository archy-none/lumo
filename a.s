import arrlen([any]): int;

type LinkList = @{ car: any, cdr: LinkList };

overload append = LinkList + LinkList;
overload from_array = [any]: LinkList;

let node(value: any) = memcpy(@{ car: value, cdr: LinkList! });
let append(self: LinkList, other: LinkList) = {
    let current = self;
    while current.cdr? loop {
        let current = current.cdr
    };
    let current.cdr = other;
    self
};
let clone(self: LinkList): LinkList = {
    let object = self.memcpy();
    if object.cdr? then {
        let object.cdr = clone(self.cdr)
    };
    object
};
let from_array(values: [any]) = {
    let list = node(values[0]);
    let length = values.arrlen();
    let index = 1;
    while index < length loop {
        let list + node(values[index]);
        let index + 1
    };
    list
};

let a = node(100);
let b = [1, 2, 3]: LinkList;
a.clone().append(b) + a
