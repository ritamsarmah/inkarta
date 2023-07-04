//
//  DetailView.swift
//  Inkarta
//
//  Created by Ritam Sarmah on 3/23/23.
//

import SwiftUI
import UniformTypeIdentifiers

struct DetailView: View {
    
    @Environment(\.dismiss) var dismiss
    
    @ObservedObject var viewModel: DetailViewModel
    
    var body: some View {
        HStack {
            Spacer()
            VStack(spacing: 4) {
                Text(viewModel.artwork.title)
                    .font(.title)
                    .fontWeight(.bold)
                    .multilineTextAlignment(.center)
                Text(viewModel.artwork.artist)
                
                Spacer()
                
                AsyncImage(url: viewModel.imageURL) { image in
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 500)
                } placeholder: {
                    ProgressView()
                        .tint(viewModel.artwork.dark ? .white : .gray)
                }
                
                Spacer()
            }
            Spacer()
        }
        .foregroundColor(viewModel.artwork.dark ? Color.white : Color.black)
        .background(viewModel.artwork.dark ? Color.black : Color.white)
        .toolbar {
            ToolbarItem {
                Button {
                    Task {
                        await viewModel.setNextId()
                    }
                } label: {
                    Image(systemName: viewModel.isNext ? "checkmark.circle.fill" : "checkmark.circle")
                }
                .disabled(viewModel.isNext)
            }
        }
        .errorAlert(info: viewModel.errorInfo)
        .environment(\.colorScheme, viewModel.artwork.dark ? .dark : .light)
    }
}

//struct DetailView_Previews: PreviewProvider {
//    static var previews: some View {
//        UploadView()
//    }
//}
