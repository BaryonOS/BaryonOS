#!/usr/bin/env python3
"""Independent Python Decimal reference; never an OS calculator backend."""
from __future__ import annotations
import argparse
import ast
from decimal import Decimal, localcontext, ROUND_HALF_EVEN
import hashlib
import json
from pathlib import Path
import random
import subprocess

QUANTUM=Decimal('0.000000001')
LIMIT=Decimal('9999999999.999999999')
class Refusal(Exception):pass

def reference(text):
    expression=text.replace(',','.')
    tree=ast.parse(expression.strip(),mode='eval')
    def bounded(value,rounded):
        fixed=value.quantize(QUANTUM,rounding=ROUND_HALF_EVEN)
        if abs(fixed)>LIMIT:raise Refusal('Range')
        return fixed,rounded or fixed!=value
    def node(n):
        if isinstance(n,ast.Constant)and type(n.value)in(int,float):return bounded(Decimal(ast.get_source_segment(expression.strip(),n)),False)
        if isinstance(n,ast.UnaryOp)and isinstance(n.op,(ast.UAdd,ast.USub)):
            value,rounded=node(n.operand);return (-value if isinstance(n.op,ast.USub)else value),rounded
        if isinstance(n,ast.BinOp):
            a,ar=node(n.left);b,br=node(n.right)
            if isinstance(n.op,ast.Add):value=a+b
            elif isinstance(n.op,ast.Sub):value=a-b
            elif isinstance(n.op,ast.Mult):value=a*b
            elif isinstance(n.op,ast.Div):
                if not b:raise Refusal('DivideByZero')
                value=a/b
            else:raise Refusal('Syntax')
            return bounded(value,ar or br)
        raise Refusal('Syntax')
    with localcontext()as context:
        context.prec=100
        try:
            value,rounded=node(tree.body)
            return f'{int(value*1000000000)} {int(rounded)}'
        except Refusal as error:return 'ERR '+str(error)

def cases():
    rng=random.Random(0x4e4f5343414c43)
    special=['9999999999.999999999','0.0000000005','0.0000000015','1.000000002500000001','0','-0','1','0.000000001']
    result=['2+3*4','(2+3)*4','1/3','0.1+0.2','-0.0000000005','0.000000003/2']
    for a in special:
        for b in special:
            for op in '+-*/':result.append(f'({a}){op}({b})')
    def generate(depth):
        if depth==0 or rng.randrange(3)==0:
            if rng.randrange(4)==0:return rng.choice(special)
            return str(rng.randrange(1000))+'.'+''.join(str(rng.randrange(10))for _ in range(rng.randrange(1,16)))
        return '('+generate(depth-1)+rng.choice('+-*/')+generate(depth-1)+')'
    result += [generate(4)for _ in range(1000)]
    return [s for s in result if len(s)<=256]

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,required=True);options=parser.parse_args()
    inputs=cases();answer=subprocess.run([str(options.binary.resolve(strict=True))],input='\n'.join(inputs)+'\n',capture_output=True,text=True,check=True,timeout=30)
    actual=answer.stdout.splitlines();assert len(actual)==len(inputs)
    for source,reply in zip(inputs,actual):
        expected=reference(source)
        assert reply==expected or expected.startswith('ERR ')and reply.startswith(expected+' '),(source,reply,expected)
    print(json.dumps({'scope':'host differential reference ONLY; native Rust library computes every result','cases':len(inputs),'passed':True,'driver_sha256':hashlib.sha256(options.binary.read_bytes()).hexdigest(),'input_sha256':hashlib.sha256(('\n'.join(inputs)+'\n').encode()).hexdigest()}))
if __name__=='__main__':main()
