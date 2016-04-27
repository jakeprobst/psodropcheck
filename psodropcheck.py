#! /usr/bin/env python

import time
import subprocess

#MEMOFFSETEP1 = 0x75d1400
#MEMOFFSETEP2 = 0x80e7b58
#MEMOFFSETEP4 = 0x75d1400
#ICOUNTOFFSET = 0x8071360
# this appears 23 bits before item drops
MAGICITEMVALUE = '\xE6\x01\x00\x55\x53\x45\x00'
MAGICITEMOFFSET = 24
MAGICCOUNTVALUE = "\x07\x48\x00\x00\x00\x55\x53\x45\x00"
MAGICCOUNTOFFSET = 0
ITEMMEMSTEP   = 0x24
AREAMEMSTEP   = 0x1B00
ICOUNTSTEP    = 0x4
MAXITEMS  = 50
AREACOUNT = 18

WEPFILE  = "wepvalues.txt"
SPECFILE = "specials.txt"

def psopid():
    return subprocess.check_output(["pgrep", "psobb.exe"]).strip()

def parsefile(path):
    f = open(path)
    out = {}
    for l in f.readlines():
        k , v = l.split(" ", 1)
        out[k] = v.strip()

    return out

items = parsefile("items.txt")
specials = parsefile("specials.txt")

def hex2str(data):
    return "%02X" % ord(data[0])
    #return hex(ord(data[0]))[2:].upper()

def val2str(data):
    out = ""
    for d in data:
        out += hex2str(d)
    return out


def printweapon(data):
    id = val2str(data[0:3])
    grind = ord(data[3])
    special = val2str([chr(ord(data[4])& 0x1F)])
    
    attr = {}
    if ord(data[6]):
        attr[ord(data[6])] = ord(data[7])
    if ord(data[8]):
        attr[ord(data[8])] = ord(data[9])
    if ord(data[10]):
        attr[ord(data[10])] = ord(data[11])

    try:
        out = items[id] + " "
    except:
        print "invalid id: " + id
        return

    if special != '00':
        out = specials[special] + " " + out
        
    if grind:
        out += "+%d " % grind

    out += "%d/" % (attr[1] if attr.has_key(1) else 0)
    out += "%d/" % (attr[2] if attr.has_key(2) else 0)
    out += "%d/" % (attr[3] if attr.has_key(3) else 0)
    out += "%d/" % (attr[4] if attr.has_key(4) else 0)
    out += "%d"  % (attr[5] if attr.has_key(5) else 0)

    print out

def printarmor(data):
    id = val2str(data[0:3])
    try:
        print items[id]
    except:
        print "invalid id: " + id

def printshield(data):
    id = val2str(data[0:3])
    try:
        print items[id]
    except:
        print "invalid id: " + id

def printmisc(data):
    id = val2str(data[0:3])
    try:
        print items[id]
    except:
        print "invalid id: " + id

def printmag(data):
    id = val2str(data[0:3])
    try:
        print items[id]
    except:
        print "invalid id: " + id

def printitem(data):
    if data[0] == '\x00':
        printweapon(data)
    elif data[0] == '\x01':
        if data[1] == '\x01':
            printarmor(data)
        elif data[1] == '\x02':
            printshield(data)
        elif data[1] == '\x03':
            printmisc(data)
    elif data[0] == '\x02':
        printmag(data)
    elif data[0] == '\x03':
        printmisc(data)


# lazy, these dont currently check if value spans multiple sections

# slow sequential search
def findmagicinrange(start, end):
    #print start, end, hex(start), hex(end)
    memoffset = -1
    countoffset = -1
    ivindex = 0
    cvindex = 0
    pid = psopid()
    f = open("/proc/" + pid + "/mem")
    f.seek(start)
    for i in xrange(start, end+1):
        try:
            val = f.read(1)
        except:
            return[-1,-1]
        if val == MAGICITEMVALUE[ivindex]:
            ivindex += 1
        else:
            ivindex = 0
        if val == MAGICCOUNTVALUE[cvindex]:
            cvindex += 1
        else:
            cvindex = 0
        if ivindex == len(MAGICITEMVALUE):
            memoffset = i-len(MAGICITEMVALUE)
            ivindex = 0
        if cvindex == len(MAGICCOUNTVALUE):
            countoffset = i
            cvindex = 0

    return [memoffset, countoffset]

# some algorithm I remember reading about but forget the name
def findmagicinrange2(f, start, end):
    mvindex = 0
    findex = start
    while findex < end:
        return
        
    
def findoffsets():
    pid = psopid()
    f = open("/proc/" + pid + "/maps")
    itemoffset = -1
    countoffset = -1
    for l in f.readlines():
        d = l.split(' ')
        print d
        if len(d[-1]) > 0 and d[-1][0] == '/': # dont need to check memmap`d files
            continue
        if not 'r' in d[1]:
            continue
        if 'stack' in d[-1]:
            continue
        start, end = d[0].split('-')
        val = findmagicinrange(int(start, 16), int(end, 16))
        print val
        if val != None:
            if val[0] != -1:
                itemoffset = val[0]
            if val[1] != -1:
                countoffset = val[1]
            if itemoffset != -1 and countoffset != -1:
                break
            
    return [itemoffset, countoffset]
    
        
def main():
    seen = set()

    print 'finding item drop offset...'
    offsets = findoffsets()
    memoff = offsets[0] + MAGICITEMOFFSET
    countoff = offsets[1] + MAGICCOUNTOFFSET
    print 'found'
    print memoff, countoff
    pid = psopid()
    while True:
        f = open("/proc/" + pid + "/mem")
        for areaoff in xrange(AREACOUNT):
            f.seek(countoff + ICOUNTSTEP * areaoff)
            count = ord(f.read(1))
            #print count
            count = 40
            if count > 127:
                continue
            for i in xrange(count):
                f.seek(memoff + AREAMEMSTEP * areaoff + ITEMMEMSTEP * i)
                data = f.read(0x24)
                if data in seen:
                    continue
                seen.add(data)
                printitem(data)

        #while len(seen) > 200:
        #    seen.pop()
        time.sleep(1)

main()















