// MOO syntax highlighting test
object TestObject
    property foo = "hello world";
    property bar(readable: true, writable: false) = 42;
    
    verb test(dobj none none)
        if (dobj == #123)
            player:tell("Found object!");
            return dobj.name + " is valid";
        elseif (dobj in {#1, #2, #3})
            for item in [1..10]
                let result = item * 2 + func(item);
                items = items + {result};
            endfor
        else
            try
                `dobj:invalid_method() ! E_VERBNF => "method not found"';
            except err (E_VERBNF)
                pass("Error handled");
            finally
                global cleanup = true;
            endtry
        endif
    endverb
endobj

// Flyweight syntax
parent = <#100, [name -> "parent"], {1, 2, 3}>;

// Complex expressions
result = (a + b) * c / d - e ^ f && g || h == i != j;

// System properties and special syntax
maxobj = $object_quota;
{x, ?y = 5, @rest} = some_list;