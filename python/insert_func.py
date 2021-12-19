from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

from argparse import ArgumentParser
from pathlib import Path
import json

from func_schema import Function, Base

def main():
    parser = ArgumentParser()
    parser.add_argument("package")
    parser.add_argument("crabfile")
    parser.add_argument("func_json")
    args = parser.parse_args()

    func_json = json.loads(args.func_json)
    package = Path(args.package[4:]) # TODO: On windows, rust gives us a weird path and we have to remove the first part. We should not do this on unix
    crabfile = Path(args.crabfile[4:])
    package_parts = package/"src"/crabfile.name
    namespace = [folder for folder in crabfile.parts if folder not in package_parts.parts]

    db_path = package/"blue.sqlite"
    db_url = f"sqlite:///{str(db_path)}"
    print(f"Using database at {db_url}")
    engine = create_engine(db_url)
    Session = sessionmaker(bind=engine)
    Base.metadata.create_all(engine)

    with Session() as session:
        session.add(Function(func_json, namespace))
        session.commit()

if __name__ == "__main__":
    main()
