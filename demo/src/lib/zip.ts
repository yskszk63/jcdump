// REF: https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT

class Reader {
  #view: DataView;
  #pos: number;

  constructor(data: ArrayBuffer) {
    this.#view = new DataView(data);
    this.#pos = 0;
  }

  get len() {
    return this.#view.byteLength;
  }

  seek(pos: number) {
    if (pos < 0) {
      pos = this.len + pos;
    }

    if (pos < 0) {
      throw new Error(`${pos} < 0`);
    }
    if (pos > this.len) {
      throw new Error(`${pos} > this.len`);
    }

    this.#pos = pos;
  }

  readU4(): number {
    if (this.#pos + Uint32Array.BYTES_PER_ELEMENT > this.len) {
      throw new Error(`${this.#pos} + ${Uint32Array.BYTES_PER_ELEMENT} > ${this.len}`);
    }
    const val = this.#view.getUint32(this.#pos, true);
    this.#pos += Uint32Array.BYTES_PER_ELEMENT;
    return val;
  }

  readU2(): number {
    if (this.#pos + Uint16Array.BYTES_PER_ELEMENT > this.len) {
      throw new Error(`${this.#pos} + ${Uint16Array.BYTES_PER_ELEMENT} > ${this.len}`);
    }
    const val = this.#view.getUint16(this.#pos, true);
    this.#pos += Uint16Array.BYTES_PER_ELEMENT;
    return val;
  }

  readBytes(len: number): Uint8Array {
    if (this.#pos + len > this.len) {
      throw new Error(`${this.#pos} + ${len} > ${this.len}`);
    }
    const val = new Uint8Array(this.#view.buffer, this.#pos, len);
    this.#pos += len;
    return val;
  }

  readString(len: number): string {
    return new TextDecoder().decode(this.readBytes(len));
  }
}

function readEndOfCentralDirectoryRecord(reader: Reader) {
  if (reader.readU4() !== 0x0605_4b50) {
    throw new Error(`Not implemented`);
  }
  const numberOfThisDisk = reader.readU2();
  const numberOfTheDiskWithTheStartOfTheCentralDirectory = reader.readU2();
  const totalNumberOfEntriesInTheCentralDirectoryOnThisDisk = reader.readU2();
  const totalNumberOfEntriesInTheCentralDirectory = reader.readU2();
  const sizeOfTheCentralDirectory = reader.readU4();
  const offsetOfStartOfCentralDirectoryWithRespectToTheStartingDiskNumber = reader.readU4();
  const dotZipFileCommentLength = reader.readU2();
  if (dotZipFileCommentLength !== 0) {
    throw new Error(`Not implemented`);
  }

  return {
    numberOfThisDisk,
    numberOfTheDiskWithTheStartOfTheCentralDirectory,
    totalNumberOfEntriesInTheCentralDirectoryOnThisDisk,
    totalNumberOfEntriesInTheCentralDirectory,
    sizeOfTheCentralDirectory,
    offsetOfStartOfCentralDirectoryWithRespectToTheStartingDiskNumber,
    dotZipFileCommentLength,
  };
}

function readCentralDirectoryEntry(reader: Reader) {
  if (reader.readU4() !== 0x02014b50) {
    throw new Error(`Central directory signature mismatch`);
  }

  const versionMadeBy = reader.readU2();
  const versionNeededToExtract = reader.readU2();
  const generalPurposeBitFlag = reader.readU2();
  const compressionMethod = reader.readU2();
  const lastModFileTime = reader.readU2();
  const lastModFileDate = reader.readU2();
  const crc32 = reader.readU4();
  const compressedSize = reader.readU4();
  const uncompressedSize = reader.readU4();
  const fileNameLength = reader.readU2();
  const extraFieldLength = reader.readU2();
  const fileCommentLength = reader.readU2();
  const diskNumberStart = reader.readU2();
  const internalFileAttributes = reader.readU2();
  const externalFileAttributes = reader.readU4();
  const relativeOffsetOfLocalHeader = reader.readU4();
  const fileName = reader.readString(fileNameLength);
  const extraField = reader.readBytes(extraFieldLength);
  const fileComment = reader.readBytes(fileCommentLength);

  return {
    versionMadeBy,
    versionNeededToExtract,
    generalPurposeBitFlag,
    compressionMethod,
    lastModFileTime,
    lastModFileDate,
    crc32,
    compressedSize,
    uncompressedSize,
    diskNumberStart,
    internalFileAttributes,
    externalFileAttributes,
    relativeOffsetOfLocalHeader,
    fileName,
    extraField,
    fileComment,
  };
}

function readLocalFileHeader(reader: Reader) {
  if (reader.readU4() !== 0x04034b50) {
    throw new Error(`Central directory signature mismatch`);
  }

  const versionNeededToExtract = reader.readU2();
  const generalPurposeBitFlag = reader.readU2();
  const compressionMethod = reader.readU2();
  const lastModFileTime = reader.readU2();
  const lastModFileDate = reader.readU2();
  const crc32 = reader.readU4();
  const compressedSize = reader.readU4();
  const uncompressedSize = reader.readU4();
  const fileNameLength = reader.readU2();
  const extraFieldLength = reader.readU2();
  const fileName = reader.readString(fileNameLength);
  const extraField = reader.readBytes(extraFieldLength);

  return {
    versionNeededToExtract,
    generalPurposeBitFlag,
    compressionMethod,
    lastModFileTime,
    lastModFileDate,
    crc32,
    compressedSize,
    uncompressedSize,
    fileName,
    extraField,
  }
}

export type ZipEntry = {
  name: string;
  open: () => ReadableStream<Uint8Array<ArrayBuffer>>;
};

export async function unzip(content: Blob): Promise<Record<string, ZipEntry>> {
  const buf = await content.arrayBuffer();

  const reader = new Reader(buf);

  // read EOCD
  reader.seek(-22);
  const eocd = readEndOfCentralDirectoryRecord(reader);

  const records: Record<string, ZipEntry> = {};
  reader.seek(eocd.offsetOfStartOfCentralDirectoryWithRespectToTheStartingDiskNumber);
  for (let i = 0; i < eocd.totalNumberOfEntriesInTheCentralDirectoryOnThisDisk; i++) {
    const cde = readCentralDirectoryEntry(reader);
    records[cde.fileName] = {
      name: cde.fileName,
      open: () => {
        reader.seek(cde.relativeOffsetOfLocalHeader);
        readLocalFileHeader(reader);
        const bytes = reader.readBytes(cde.compressedSize);
        const rawStream = new Blob([new Uint8Array(bytes)]).stream();
        switch (cde.compressionMethod) {
          // The file is stored (no compression)
          case 0:
            return rawStream;

          // The file is Deflated
          case 8:
            return rawStream.pipeThrough(new DecompressionStream("deflate-raw"));

          default:
            throw new Error(`Not supported: ${cde.compressionMethod}`);
        }
      },
    };
  }

  return records;
}
