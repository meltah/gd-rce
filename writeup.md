# Remote Code Execution for Geometry Dash 2.1

## Discovery

The vulnerability was discovered in June of 2022 by a group of developers in the Geode SDK server (alk1m123, camila314, cos8o). By August 2022 I had developed the full exploit.

## The Vulnerability

Geometry Dash has a game mechanic called pickup triggers, which modify an array each time they are triggered (usually by being passed by the player, but triggers can trigger other triggers as is used in this exploit). The function responsible is this (RVA `0x111890`):

```c++
void GJBaseGameLayer::collectItem(int itemID, int amount) {
	bool decreasing = amount < 0;
	if(amount < 0) {
		amount = -amount;
	}
	for(; amount != 0; amount--) {
		if(decreasing) {
			this->m_effectManager->m_itemValues[itemID]--;
		} else {
			this->m_effectManager->m_itemValues[itemID]++;
		}
		// ...
	}
	// ...
}
```

`m_itemValues` is an `int[1100]`, but the `itemID` argument isn't bounds-checked; an out-of-bounds vulnerability. The item ID is actually checked in the UI when placing down a pickup trigger object and entering the number, but this can be easily bypassed by modifying the level data programatically.

## Tools

- We can perform arbitrary addition and subtraction on a memory location relative to the `m_effectManager` of the `GJBaseGameLayer` object.
- By combining triggers which can perform comparisons, we can zero out a location, then add or subtract, giving us the ability to assign a value as well.
- Using an intricate setup of triggers, we can trigger something based on the amount of frames that have passed.

## The Exploit

(note: Geometry Dash is a 32-bit game, so addresses are 4 bytes)

First, we need a way to call arbitrary functions. For that, we can perform a vtable reuse attack. The vtable of `GJEffectManager` (the type of `m_effectManager`) is at offset zero, and `m_itemValues` is at offset `0x9f0`. Divide that by 4 and we get an item ID of `-636` to modify the vtable. ASLR isn't an issue here, as the vtable will always be at the same offset from the executable's base, wherever that may be, so all we have to do is add or subtract.

I tried several approaches, including manually crafting a ROP in a scratch location then calling it, but because the primitive I discovered for writing to an arbitrary address can only write one byte per frame, it was pretty slow. I was going to settle with that but one day I found this piece of assembly in the game's executable, while I was looking for better ways to perform the exploit (RVA `0x194a5c`):

```x86asm
xchg esp, esi
add byte [eax], al
add bh, bh
adc eax, 0x682d60
mov al, 0x1
pop edi
ret
```

This sequence of instructions is actually in the *middle* of other instructions, hence the weird nonsensical operations being performed. What's important here is the `xchg esp, esi`, as it lets us set the stack pointer to `esi`, allowing us to perform a ROP. Now, this isn't special, there are many `xchg esp, REG`s in the executable, but we'll get to that.

To use this gadget, we need to be able to control `esi`. Luckily, there is a function `CCCallFuncN::execute`, which does this:

```x86asm
push esi
mov  esi, ecx

mov  eax, dword ptr [esi + 0x3c]
test eax, eax
jz   LAB_10086c22

push dword ptr [esi + 0x24]
mov  ecx, dword ptr [esi + 0x34]
call eax

...
```

As you can see, it moves `ecx` into `esi`, and `ecx` is the `this` parameter, which will be passed in any virtual function invocation. After that, it loads a function pointer from `this + 0x3c`, then pushes an argument from `this + 0x24` and sets the `this` parameter to the value at `this + 0x34`, then calls the function pointer it loaded. We can ignore the argument and `this` parameter for now. We just need to set `this + 0x3c` (which is at item ID `-621`) to the gadget from above, and then `esi` will be equal to `this`, which means we will *turn our `GJEffectManager`* - the thing we can modify any value in trivially - *into a stack*.

There is one problem: the `CCCallFuncN::execute` function isn't actually in the game's executable. It's in `libcocos2d.dll`, the game engine Geometry Dash uses. To reach this, there's actually a clever method we can use: there are some function pointers to `libcocos2d.dll` in the executable, so if we can find something which will modify the vtable for us, we're set. A function which modifies the vtable is usually a constructor, and indeed, the `CCIMEDelegate` constructor is perfect: it sets `this->vtable = CCIMEDelegate::vftable` and almost nothing else. To call this constructor, we can set our vtable to `0x282750`, the RVA of the function pointer to that external function in the Geometry Dash executable, minus an offset of `0x16C`, which is the offset of the `update` function in the `GJEffectManager` vtable. `update` is a function that's called every frame, but it happens to be a no-op (it just draws the children of the node, but `GJEffectManager` has no children). Nothing else in the vtable is used. That means we can just add `0x282750` minus the current vtable value, `0x2a5fc4`, minus `0x16C` to the vtable so it equals `GD+0x282750 - 0x16C`, then the `update` call will add back the `0x16C` to get `GD+0x282750`, then it will dereference that to get the function pointer to the `CCIMEDelegate` constructor and execute it.

Then, we wait one frame, to allow `update` to execute, then the frame after that we continue with the exploit. We can wait a frame using a specific trigger setup which can trigger more pickup triggers (and thus more `collectItem` calls) after a given frame delay.

Our vtable is now in `libcocos2d.dll`'s address space. We can now simply add the RVA of `CCCallFuncN::execute` (`0x11F910`) minus the `CCIMEDelegate` vtable's RVA (`0x12FCFC`), and again minus `0x16C` to account for the position of the `update` function.

The next frame, `CCCallFuncN::execute` will execute, so let's make sure our fields are set right. We want to set `this + 0x3c`, the function pointer to be executed by `CCCallFuncN::execute`, to our stack pivot gadget, so that `esi` will be set to `this` then `xchg esp, esi` will be executed.

Now to address a few things:

### Why do we need `CCCallFuncN::execute`?

A few reasons:

- We're going to use the fact that it can pass arguments in a moment (`update` takes no arguments except `this`).
- If we didn't, every function we wanted to call would have to have a **function pointer** to it in the executable, not just *be* in the executable. `CCCallFuncN::execute` removes one layer of indirection.

### If `CCCallFuncN::execute` can call any function directly, why do we need to do a stack pivot in the first place?

My previous approaches didn't use a stack pivot for this reason. However, it ended up being too difficult to do anything meaningful by *just* calling functions using `CCCallFuncN::execute` (for example, the amount of arguments is limited, and we can't do anything with the return values). In fact, I used a different function, `CCCallFuncND::execute`, as this provided 2 arguments instead of 1, but it was still necessary to construct a ROP chain and then call that. Constructing the ROP chain was done by calling a function, `setPosition`, and giving it a `this` argument pointing to some scratch space (this could be guaranteed to point to valid space by copying over the vtable field to the field specifying the new `this` value - using triggers - then offsetting it to point to the data section), and passing it arguments which would modify some fields of `this`. However, this had a maximum speed of 8 bytes per frame theoretically, but closer to 1 or 2 bytes in practice, because I couldn't modify the argument fields that much due to the fact that `collectItem` has an `O(n)` loop (this is just bad code by the developer, it can easily be done in `O(1)`) where `n` is the amount.

### Why are we using this weird pivot which modifies `[eax]` instead of something cleaner?

Indeed, there are cleaner stack pivots in the executable, such as this (`0x1330f`):

```x86asm
xchg eax, esp
ret
```

Assuming we used another gadget first to set `eax` to `esi`, we would run into the issue that **this returns directly into the new stack**, which in this case would set `eip = vtable` (!!). This would instantly segfault. The "weird pivot" is the one of the only ones I found which performs a **pop** before returning, which means `esp` will be offsetted by 4, allowing the next field after the vtable (which we can control) to be the start of the ROP chain.

---

Alright, moving on. To set `this + 0x3c` to the stack pivot at RVA `0x194a5c`, we need to copy over some value with a known offset from the base to item ID `-621` then add or subtract. We had such a value at the beginning - the `GJEffectManager` vtable! So, before modifying it at all, we must simply copy the value (using a combination of comparison and pickup triggers) at ID `-636` to ID `-621`. Then, we add `0x194a5c` minus the `GJEffectManager` vtable (`0x2a5fc4`, remember, this is all before modification of the vtable).

If all goes well, this should set the stack pointer to our effect manager. If we try this though, we'll get a segfault. Remember the code of `CCCallFuncN::execute`?

```x86asm
...

mov  eax, dword ptr [esi + 0x3c]

...

call eax ; <----

...
```

`eax` is the stack pivot! But in the stack pivot itself, we execute `add byte [eax], al`. That's a modification of an executable section, and will segfault. The solution to this is to make use of the fact that `CCCallFuncN::execute` lets us pass arguments, which will let us execute one more gadget before our stack pivot. If we set the function to execute to be a gadget like this (RVA `0x76ba`):

```x86asm
mov eax, ecx
pop ebp
ret
```

`ecx`, which is controlled by the `this` parameter, we can set that too! Let's set it to some random scratch space in the executable (RVA `0x31e08c`), using the same copy-and-offset method as we did for the function pointer itself. The `pop ebp` simply pops off the return value pushed by the `call` - we don't want that. Then the `ret` will return into the next value on the stack, which happens to be the argument in this calling convention! We can set our *argument* to the stack pivot, and now that we've set `eax` to writable memory, this won't segfault!

## Getting Arbitrary Code Execution

At this point, we can construct a ROP chain to do pretty much anything. There's one limitation though: all of this is happening in the `GJEffectManager`, which has a limited size, and we don't want to be overriding the entire struct with garbage, as it's used in the game. Here's what I did:

Setting the field right after the vtable (item ID `-635`) will let us control `eip`. Let's set it to this (`0x16160b`):

```x86asm
ret 0xa74
```

This is a `ret` instruction which pops `0xa74` bytes off the stack after returning to a function of our choice which is at item ID `-634`. That means we can skip all of the important data in the struct and land right in our `m_itemValues` array, which is explicitly meant for us to modify the data in. Our stack pointer will end up at `m_effectManager + 12 + 0x174` (item ID `36`! not out-of-bounds). Now, we have around a thousand item IDs before we run out of space. But there's another problem: modifying these item IDs can only occur using the `collectItem` function, which is `O(n)`. In order to not have the game freeze for a while when performing the exploit, we should try to modify the item IDs by as little an amount as possible. I managed to golf this down to modifying just **nine** item IDs, which is completely unnoticeable on my system. All 9 of these are relative offsets to the Geometry Dash executable's base address, so they must be done using the copy-and-offset method.

To achieve arbitrary code execution using just nine item IDs, I used this ROP as essentially a "bootloader" for *another* ROP, stored somewhere else. Here's the ROP, starting at item ID `36`, and with the function at `-634` being `GameManager::sharedState` (RVA `0xc4a50`), which takes no arguments and simply returns the shared game manager instance in `eax`:

```
// eax = GameManager*

36: GD + 0x3400d        // pop edi; ret
37: GD + 0x281c9e       // add esp, 8; ret

38: GD + 0x117f7        // mov ecx, eax; call edi
39:                     // skipped by add esp, 8
40: GD + 0x10e5df       // mov eax, dword ptr [ecx + 0x164]; pop ebp; ret 0xc
41:                     // ebp
42: GD + 0x17f83        // mov ebx, eax; call edi
43:                     // 4
44:                     // 8
45:                     // c
46:                     // skipped by add esp, 8
47: GD + 0x1fc4f4       // mov ecx, dword ptr [ebx + 0x22c]; call edi
48:                     // skipped by add esp, 8
49: GD + 0xca8fe        // mov ecx, dword ptr [ecx + 0x118]; call edi
50:                     // skipped by add esp, 8

// ecx now has guidelines string, make it the stack
51: GD + 0x7f3cb        // push ecx; pop ebp; add byte ptr [eax], al; push eax; mov ecx, esi; call edi
// no skip here, it skips pushed eax
52: GD + 0x1089         // mov esp, ebp; pop ebp; ret
```

Let's go through what it's doing.

- (36) First we pop `edi`, which will set it to the value at item ID `37`, which is a pointer to an `add esp, 8; ret` gadget. This is done because many other gadgets down the line end with a `call edi`, and we want those calls to simply ignore the return address that was pushed and continue with the ROP.
- (38) Then, we move `eax`, which contains the `GameManager` pointer, into `ecx`, so that the next gadget can access the value at `[ecx + 0x164]`. This is the `m_playLayer` field of `GameManager`, which contains a `PlayLayer*`. At 42 we then move that into `ebx`.
- (47) We then access the value at `[ebx + 0x22c]`, which is the `m_levelSettings` field of `PlayLayer`. This contains a `LevelSettingsObject*`.
- (49) We then access the value at `[ecx + 0x118]`, which is the `m_guidelineString` field of `LevelSettingsObject`. The type of this field is `std::string`, and at offset 0 of that happens to be the `char*` containing the data.
- (51) The important instructions here are `push ecx; pop ebp`. The rest is just noise that came with the instructions. It's mostly harmless, except the `add byte ptr [eax], al`, which must be corrected later on when we have ACE.
- (52) Make the guideline string our new stack!

The `m_guidelineString` field is *supposed* to contain information about the song, in a specific format, which draws lines ("guidelines") in the level editor corresponding to the music so that level creators can sync their levels to the music. However, we can put anything in here as long as it doesn't contain null bytes, and it'll load the level fine. However, if it doesn't conform to the format, it will crash the level editor if opened. I see this as a benefit, because curious folks can no longer open the level in the editor to try to get an idea of how the exploit works :^)

We can modify the guideline string easily by modifying the level data. However, other than null bytes, we also can't have two additional characters: commas (`,`) and semicolons (`;`), as these mark the *end* of a field in the level data (it's a custom textual format), which would cut our guideline string short.

We now move on to the second part of the ROP.

This one is over 200 lines, so I won't explain every detail of it. You can check the [code](https://github.com/meltah/gd-rce) yourself. But here's the gist of it:

Since this is now a static string, we *can't* use offsets relative to the executable base anymore, so we have to find something with ASLR disabled. Luckily, `glew32.dll` doesn't have ASLR on, so we can use its addresses directly. We didn't do this from the start because the addresses are huge, and `collectItem` is `O(n)`. Plus, `glew32` has way less useful gadgets anyways.

First, I made a nop sled (just a bunch of pointers to `ret`s) that's two million long. This gives 8mb of space above us - given that this is now the stack, we want to make sure Windows functions won't overflow it.

Then, I load the guideline string *again* from `PlayLayer`. We need this so we can self-modify the stack; some values *must* be zeroed out, such as the address for a `VirtualAlloc` call. We can't just put zeroes there because we're not allowed to have null bytes, so instead we put a zero value there using the ROP itself.

I then zero out the `lpAddress` parameter of the incoming `VirtualAlloc` call, set the correct `flAllocationType` and `flProtect` parameters, zero out unwanted bytes of the length parameter for the base64 decode function (we will use this to load the payload, as it can't have zeroes), add a pointer to a `"libcocos2d"` string for a `GetModuleHandleA` call and the payload string for the base64 call, and finally call `VirtualAlloc` to allocate space for our payload, which is both writable and executable.

The self-modification of strings part was interesting because it involved an [Integer Linear Programming](https://en.wikipedia.org/wiki/Integer_programming) problem: to offset the guideline string pointer by a certain amount, that in itself uses up 4 bytes of space before the string. I used z3 to find the optimal solution, although it can likely be done without it.

After the `VirtualAlloc`, we use a gadget to store its return value in both the output parameter for the base64 call and its return address. We then call `GetModuleHandleA` to get `libcocos2d.dll`'s base address, which we then offset to get the address of a base64 decode function. The input to the base64 decoding is the payload string, and the output is the allocated block. After the decoding is done, we simply return right into our allocated block containing whatever code we like.
